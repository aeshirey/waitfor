use crate::{err::Error, WaitMultiple};
use std::{
    cell::{Cell, RefCell},
    fs::Metadata,
    path::PathBuf,
    time::{Duration, Instant, SystemTime},
};
use url::Url;

#[derive(Debug, Clone)]
pub enum Wait {
    Elapsed {
        end_instant: Instant,
    },
    Exists {
        not: bool,
        path: PathBuf,
    },
    HttpGet {
        not: bool,
        url: String,
        status: u16,
    },
    TcpHost {
        not: bool,
        host: String,
    },
    Update {
        not: bool,
        path: PathBuf,
        modified: Cell<Option<SystemTime>>,
    },
    FileSize {
        not: bool,
        path: PathBuf,
        bytes: Cell<Option<u64>>,
    },
    // FileOpen(??), // Check if a handle is open on a particular file (ie, when a file is done being modified)
}

impl Wait {
    pub fn elapsed(duration: Duration) -> Self {
        Self::Elapsed {
            end_instant: Instant::now().checked_add(duration).unwrap(),
        }
    }

    pub fn elapsed_str(elapsed: &str) -> Result<Self, Error> {
        match parse_duration(elapsed) {
            Ok(duration) => Ok(Self::elapsed(duration)),
            Err(_) => Err(Error::InvalidDuration(elapsed.to_string())),
        }
    }

    pub fn exists<T: Into<PathBuf>>(path: T) -> Self {
        Self::Exists {
            not: false,
            path: path.into(),
        }
    }

    pub fn not_exists<T: Into<PathBuf>>(path: T) -> Self {
        Self::Exists {
            not: true,
            path: path.into(),
        }
    }

    pub fn update<T: Into<PathBuf>>(path: T) -> Result<Self, Error> {
        let (path, _) = Self::get_metadata(path.into())?;

        Ok(Self::Update {
            not: false,
            path,
            modified: None.into(),
        })
    }

    pub fn not_update<T: Into<PathBuf>>(path: T) -> Result<Self, Error> {
        let (path, _) = Self::get_metadata(path.into())?;

        Ok(Self::Update {
            not: true,
            path,
            modified: None.into(),
        })
    }

    pub fn http_get(urlarg: &str) -> Self {
        let (status, url) = parse_http_get(urlarg);
        Self::HttpGet {
            not: false,
            url,
            status,
        }
    }

    pub fn not_http_get(urlarg: &str) -> Self {
        let (status, url) = parse_http_get(urlarg);
        Self::HttpGet {
            not: true,
            url,
            status,
        }
    }

    pub fn tcp(host: &str) -> Result<Self, Error> {
        let host = host.to_string();
        if validate_tcp(&host) {
            Ok(Self::TcpHost { not: false, host })
        } else {
            Err(Error::InvalidHost(host))
        }
    }

    pub fn not_tcp(host: &str) -> Result<Self, Error> {
        let host = host.to_string();
        if validate_tcp(&host) {
            Ok(Self::TcpHost { not: true, host })
        } else {
            Err(Error::InvalidHost(host))
        }
    }

    pub fn file_size<T: Into<PathBuf>>(path: T) -> Result<Self, Error> {
        let (path, _) = Self::get_metadata(path.into())?;

        Ok(Self::FileSize {
            not: false,
            path,
            bytes: None.into(),
        })
    }

    pub fn not_file_size<T: Into<PathBuf>>(path: T) -> Result<Self, Error> {
        let (path, _) = Self::get_metadata(path.into())?;

        Ok(Self::FileSize {
            not: true,
            path,
            bytes: None.into(),
        })
    }

    fn get_metadata(path: PathBuf) -> Result<(PathBuf, Metadata), Error> {
        let metadata = path
            .metadata()
            .map_err(|_| Error::MetadataUnavailable(path.clone()))?;

        if !metadata.is_file() || metadata.modified().is_err() {
            Err(Error::InputNotFile(path))
        } else {
            Ok((path, metadata))
        }
    }

    pub fn condition_met(&self) -> bool {
        match self {
            Wait::Elapsed { end_instant } => *end_instant < Instant::now(),
            Wait::Exists { not: true, path } => !path.exists(),
            Wait::Exists { not: false, path } => path.exists(),
            Wait::HttpGet { not, url, status } => {
                let result = ureq::get(url).call();
                if *not {
                    *status != result.status()
                } else {
                    *status == result.status()
                }
            }
            Wait::TcpHost { not: false, host } => std::net::TcpStream::connect(host).is_ok(),
            Wait::TcpHost { not: true, host } => std::net::TcpStream::connect(host).is_err(),
            Wait::Update {
                not: false,
                path,
                modified,
            } => {
                let modified_time = path.metadata().ok().map(|m| m.modified().ok()).flatten();
                match (modified.get(), modified_time) {
                    // Can't get the modified time
                    (_, None) => false,
                    // Times are different -- condition is met
                    (Some(prev), Some(curr)) if prev != curr => true,
                    // All other cases -- save this time
                    (_, curr) => {
                        modified.set(curr);
                        false
                    }
                }
            }
            Wait::Update {
                not: true,
                path,
                modified,
            } => {
                let modified_time = path.metadata().ok().map(|m| m.modified().ok()).flatten();
                match (modified.get(), modified_time) {
                    // Can't get the modified time
                    (_, None) => false,
                    // Times are equal -- condition is met
                    (Some(prev), Some(curr)) if prev == curr => true,
                    // All other cases -- save this time
                    (_, curr) => {
                        modified.set(curr);
                        false
                    }
                }
            }

            Wait::FileSize {
                not: false,
                path,
                bytes,
            } => {
                let file_size = path.metadata().map(|m| m.len()).ok();
                match (bytes.get(), file_size) {
                    // Can't get the file size. This is probably due to file non-existence,
                    // so we'll assume the condition is met
                    (_, None) => true,
                    // Sizes are different -- condition is met
                    (Some(prev), Some(curr)) if prev != curr => true,
                    // First time or subsequent with equal values - save the size and try again
                    (_, curr) => {
                        bytes.set(curr);
                        false
                    }
                }
            }
            Wait::FileSize {
                not: true,
                path,
                bytes,
            } => {
                let file_size = path.metadata().map(|m| m.len()).ok();
                match (bytes.get(), file_size) {
                    // Can't get the file size. This is probably due to file non-existence,
                    // so we'll assume the condition is met
                    (_, None) => true,
                    // Size hasn't changed -- condition is met
                    (Some(prev), Some(curr)) if prev == curr => true,
                    // First time or subsequent with changing values - save the (new) size and try again
                    (_, curr) => {
                        bytes.set(curr);
                        false
                    }
                }
            }
        }
    }
}

impl std::ops::BitOr for Wait {
    type Output = WaitMultiple;

    fn bitor(self, rhs: Self) -> Self::Output {
        let v = vec![self, rhs];
        WaitMultiple(v)
    }
}

/// Tries to parse a URL using the `url` crate.
fn parse_url(urlarg: &str) -> Result<Url, Error> {
    let violations = RefCell::new(Vec::new());
    Url::options()
        .syntax_violation_callback(Some(&|v| {
            violations.borrow_mut().push(v);
        }))
        .parse(urlarg)
        .map_err(|_| Error::InvalidUrl(urlarg.to_string()))
}

/// Parses an input argument for an HTTP GET into the expected status code and URL to hit.
///
/// The URL is validated with the `url` crate, if possible, cleaning potential errors.
/// If that fails, the URL is used as-is.
fn parse_http_get(urlarg: &str) -> (u16, String) {
    let urlbytes = urlarg.chars().collect::<Vec<_>>();

    let (status_code, urlarg) = if urlarg.len() > 4
        && urlbytes[0..3].iter().all(|c| c.is_numeric())
        && urlbytes[3] == ','
    {
        let code = 100 * (urlbytes[0] as u16 - '0' as u16)
            + 10 * (urlbytes[1] as u16 - '0' as u16)
            + (urlbytes[2] as u16 - '0' as u16);

        (code, &urlarg[4..])
    } else {
        (200, urlarg)
    };

    if let Ok(url) = parse_url(urlarg) {
        (status_code, url.to_string())
    } else {
        (status_code, urlarg.to_string())
    }
}

/// Verifies that the specified host or address contains a numeric port number.
pub fn validate_tcp(hostarg: &str) -> bool {
    // Assume that the last location of ':' is the delimiter for the port
    let last_colon = hostarg.char_indices().filter(|(_i, c)| c == &':').last();
    if let Some((i, _c)) = last_colon {
        // Everything after the colon should be a numeric port number
        hostarg.chars().skip(i + 1).all(|c| c.is_numeric())
    } else {
        // There's no ':' in the input, so assume this isn't a host to which we can connect
        false
    }
}

/// Parses a simple human-readable duration, returning a `Duration`
///
/// "3h10m" -> 11400 seconds
pub fn parse_duration(duration: &str) -> Result<Duration, String> {
    let mut total_delay = 0;

    let mut acc = 0;
    for c in duration.chars() {
        match c {
            '0'..='9' => {
                acc *= 10;
                acc += c.to_digit(10).unwrap();
            }
            'd' => {
                // days
                total_delay += acc * 86400;
                acc = 0;
            }
            'h' => {
                // hours
                total_delay += acc * 3600;
                acc = 0;
            }
            'm' => {
                // minutes
                total_delay += acc * 60;
                acc = 0;
            }
            's' => {
                // seconds
                total_delay += acc;
                acc = 0;
            }
            _ => return Err(format!("Invalid duration definition: {}", duration)),
        }
    }

    total_delay += acc;

    let d = Duration::from_secs(total_delay as u64);

    Ok(d)
}

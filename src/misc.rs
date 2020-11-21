use std::cell::RefCell;
use std::time::Duration;
use url::Url;

/// Parses a simple human-readable duration, returning a `Duration`
///
/// "3h10m" -> 11400 seconds
pub fn parse_duration(duration: &str) -> Result<Duration, ()> {
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
            },
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
            _ => return Err(()),
        }
    }

    total_delay += acc;

    let d = Duration::from_secs(total_delay as u64);

    Ok(d)
}

/// Parses an input argument for an HTTP GET into the expected status code and URL to hit.
///
/// The URL is validated with the `url` crate, if possible, cleaning potential errors.
/// If that fails, the URL is used as-is.
pub fn parse_http_get(urlarg: &str) -> (u16, String) {
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

    /*
    (status_code, urlarg.to_string())
    */
}

/// Tries to parse a URL using the `url` crate.
fn parse_url(urlarg: &str) -> Result<Url, ()> {
    let violations = RefCell::new(Vec::new());
    let url = Url::options()
        .syntax_violation_callback(Some(&|v| {
            violations.borrow_mut().push(v);
        }))
        .parse(urlarg)
        .map_err(|_| ())?;

    Ok(url)
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_get_pass() {
        assert_eq!(
            (200, "http://google.com/".to_string()),
            parse_http_get("200,http://google.com")
        );

        assert_eq!(
            (200, "http://google.com/".to_string()),
            parse_http_get("http://google.com")
        );

        assert_eq!(
            (404, "http://does-not-exist.com/".to_string()),
            parse_http_get("404,http://does-not-exist.com")
        );

        assert_eq!(
            (200, "http://example.com/".to_string()),
            parse_http_get("200,http:/example.com/")
        );

        assert_eq!(
            (200, "http://example.com/?foo=bar".to_string()),
            parse_http_get("200,http:/example.com/?foo=bar")
        );

        assert_eq!(
            (200, "a string that isn't a url".to_string()),
            parse_http_get("a string that isn't a url")
        );
    }

    #[test]
    fn validate_hosts() {
        // Loopback with and without ports -- one is good, one not
        assert!(validate_tcp("127.0.0.1:22"));
        assert!(!validate_tcp("127.0.0.1"));

        // Or a domain name
        assert!(validate_tcp("google.com:80"));
        assert!(!validate_tcp("google.com"));

        // Longer port
        assert!(validate_tcp("localhost:5000"));

        // IPv6 should still work since the port is at the end
        assert!(validate_tcp("[2001:db8::1]:8080"));

        // Note that this simplistic approach doesn't properly handle everything.
        // Ideally, this one would fail because it doesn't specify a port:
        //assert!(!validate_tcp("[2001:db8::1]"));
        // There are multiple ways IPv6 addresses specify ports, so this remains a TODO.
        // See the RFC for details on IPv6 ports: https://tools.ietf.org/html/rfc5952#page-11
    }
}

use std::path::Path;
use std::time::Instant;

#[derive(Debug)]
pub enum Wait {
    Elapsed { end_instant: Instant },
    Exists { not: bool, path: String },
    HttpGet { not: bool, url: String, status: u16 },
    TcpHost { not: bool, host: String },
    Pid { pid: u64 },
    // FileOpen(??), // Check if a handle is open on a particular file (ie, when a file is done being modified)
}

impl Wait {
    pub fn condition_met(&self) -> bool {
        match self {
            Wait::Elapsed { end_instant } => *end_instant < Instant::now(),
            Wait::Exists { not: true, path } => !Path::new(path).exists(),
            Wait::Exists { not: false, path } => Path::new(path).exists(),
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
            Wait::Pid { pid: _ } => todo!(),
        }
    }
}

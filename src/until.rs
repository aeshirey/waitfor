use std::path::Path;
use std::time::Instant;

#[derive(Debug)]
pub enum Wait {
    Elapsed { end_instant: Instant },
    Exists { not: bool, path: String },
    HttpGet { url: String, status: u16 },
    Pid(u64),
    // FileOpen(??), // Check if a handle is open on a particular file (ie, when a file is done being modified)
}

impl Wait {
    pub fn condition_met(&self) -> bool {
        match self {
            Wait::Elapsed { end_instant } => *end_instant < Instant::now(),
            Wait::Exists { not, path } if *not => !Path::new(path).exists(),
            Wait::Exists { not: _, path } => Path::new(path).exists(),
            Wait::HttpGet { url, status } => {
                let result = ureq::get(url).call();
                *status == result.status()
            }
            Wait::Pid(_pid) => todo!(),
        }
    }
}

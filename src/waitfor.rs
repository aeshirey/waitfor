use std::{
    cell::Cell,
    path::Path,
    time::{Instant, SystemTime},
};

use crate::misc;

#[derive(Debug)]
pub enum Wait {
    Elapsed {
        end_instant: Instant,
    },
    Exists {
        not: bool,
        path: String,
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
        path: String,
        modified: Cell<Option<SystemTime>>,
    },
    FileSize {
        not: bool,
        path: String,
        bytes: Cell<Option<u64>>,
    },
    Pid {
        pid: u64,
    },
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
            Wait::Update {
                not: false,
                path,
                modified,
            } => {
                match (modified.get(), misc::get_modified_time(path)) {
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
                match (modified.get(), misc::get_modified_time(path)) {
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
                match (bytes.get(), misc::get_file_size(path)) {
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
                match (bytes.get(), misc::get_file_size(path)) {
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

            Wait::Pid { pid: _ } => todo!(),
        }
    }
}

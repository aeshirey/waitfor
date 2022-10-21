use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    MetadataUnavailable(PathBuf),
    InputNotFile(PathBuf),
    InvalidUrl(String),
    InvalidHost(String),
    InvalidDuration(String),
}

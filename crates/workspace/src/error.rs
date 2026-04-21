use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("git error: {0}")]
    Git(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

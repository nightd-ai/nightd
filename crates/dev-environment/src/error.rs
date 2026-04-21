use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("command failed: {0}")]
    Command(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

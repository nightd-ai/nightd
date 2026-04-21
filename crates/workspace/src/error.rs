use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("jujutsu command failed: {message}\nstdout: {stdout}\nstderr: {stderr}")]
    JujutsuCommand {
        message: String,
        stdout: String,
        stderr: String,
    },
    #[error("workspace name collision after max retries")]
    NameCollision,
    #[error("source repository is not a valid jujutsu repo: {0}")]
    InvalidSourceRepo(String),
    #[error("could not resolve app data directory")]
    NoAppDataDir,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

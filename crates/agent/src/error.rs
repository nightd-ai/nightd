use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("ACP error: {0}")]
    Acp(String),
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

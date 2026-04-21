use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("agent error: {0}")]
    Agent(#[from] agent::Error),
    #[error("workspace error: {0}")]
    Workspace(#[from] workspace::Error),
    #[error("dev environment error: {0}")]
    DevEnvironment(#[from] dev_environment::Error),
    #[error("session not found: {0}")]
    SessionNotFound(uuid::Uuid),
}

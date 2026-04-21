//! Error types for the agent crate.

use thiserror::Error;

/// Result type alias for agent operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Error types that can occur in agent operations.
#[derive(Error, Debug)]
pub enum Error {
    /// Database error.
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    /// Configuration error.
    #[error("configuration error: {0}")]
    Config(String),

    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(#[from] toml::ser::Error),

    /// Deserialization error.
    #[error("deserialization error: {0}")]
    Deserialization(#[from] toml::de::Error),

    /// IO error.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Session not found.
    #[error("session not found: {0}")]
    SessionNotFound(uuid::Uuid),

    /// Invalid session state transition.
    #[error("invalid session state transition from {from} to {to}")]
    InvalidStateTransition { from: String, to: String },

    /// ACP protocol error.
    #[error("ACP error: {0}")]
    Acp(String),

    /// Migration error.
    #[error("migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
}

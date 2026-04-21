//! Agent crate for managing AI agent sessions and operations.
//!
//! This crate provides functionality for creating, managing, and tracking
//! agent sessions with persistent storage via SQLite.

pub mod api;
pub mod config;
pub mod db;
pub mod error;
pub mod models;
pub mod scheduler;
pub mod session;
pub mod types;

pub use error::{Error, Result};
pub use models::{SessionStatus, SessionStatusDetail};
pub use scheduler::{Component, Scheduler};
pub use session::SessionHandle;
pub use types::{AgentId, SessionRequest};

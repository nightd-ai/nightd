//! Public API for agent operations.
//!
//! This module exposes the high-level interface for interacting with agents.

use crate::{Result, session::SessionManager};

/// Main API client for agent operations.
pub struct AgentClient {
    session_manager: SessionManager,
}

impl AgentClient {
    /// Create a new agent client with the given database URL.
    pub async fn new(database_url: &str) -> Result<Self> {
        let session_manager = SessionManager::new(database_url).await?;
        Ok(Self { session_manager })
    }

    /// Get a reference to the session manager.
    pub fn sessions(&self) -> &SessionManager {
        &self.session_manager
    }
}

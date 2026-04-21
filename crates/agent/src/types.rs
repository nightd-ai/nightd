//! Core public API types for agent operations.

use std::path::PathBuf;

/// Unique identifier for an agent.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct AgentId(pub String);

impl AgentId {
    /// Create a new agent ID from a string.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for AgentId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for AgentId {
    fn from(s: &str) -> Self {
        Self(s.to_owned())
    }
}

impl AsRef<str> for AgentId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Request to create a new agent session.
#[derive(Debug, Clone)]
pub struct SessionRequest {
    /// Workspace path for the session.
    pub workspace: PathBuf,
    /// Prompt or task description.
    pub prompt: String,
    /// Agent identifier.
    pub agent: AgentId,
    /// Priority level (higher = more important).
    pub priority: i32,
}

impl SessionRequest {
    /// Create a new session request with the given workspace and prompt.
    pub fn new(workspace: impl Into<PathBuf>, prompt: impl Into<String>) -> Self {
        Self {
            workspace: workspace.into(),
            prompt: prompt.into(),
            agent: AgentId::new("default"),
            priority: 0,
        }
    }

    /// Set the agent ID for this request.
    pub fn agent(mut self, agent: impl Into<AgentId>) -> Self {
        self.agent = agent.into();
        self
    }

    /// Set the priority for this request.
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

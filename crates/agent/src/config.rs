//! Configuration for the agent crate.

use serde::{Deserialize, Serialize};

/// Agent configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Database URL or path.
    pub database_url: String,
    /// Default agent identifier.
    pub default_agent_id: String,
    /// Maximum number of concurrent sessions.
    pub max_concurrent_sessions: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_url: "sqlite://agent.db".to_string(),
            default_agent_id: "default".to_string(),
            max_concurrent_sessions: 4,
        }
    }
}

impl Config {
    /// Load configuration from a TOML file.
    pub fn from_file(path: impl AsRef<std::path::Path>) -> crate::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to a TOML file.
    pub fn to_file(&self, path: impl AsRef<std::path::Path>) -> crate::Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

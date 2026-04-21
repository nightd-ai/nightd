use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "snake_case")]
pub enum SessionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub agent_id: String,
    pub model: Option<String>,
    pub repo_url: String,
    pub branch: String,
    pub prompt: String,
    pub init_command: Option<String>,
    pub status: SessionStatus,
    pub result: Option<String>,
    pub error: Option<String>,
    pub priority: i32,
    pub created_at: OffsetDateTime,
    pub started_at: Option<OffsetDateTime>,
    pub finished_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub id: String,
    pub command: std::path::PathBuf,
    pub args: Vec<String>,
    pub env: std::collections::HashMap<String, String>,
    pub model_flag: Option<String>,
}

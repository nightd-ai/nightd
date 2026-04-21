//! Data models for agent sessions and operations.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

/// Session status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "snake_case")]
pub enum SessionStatus {
    /// Session is pending and waiting to be started.
    Pending,
    /// Session is currently running.
    Running,
    /// Session completed successfully.
    Completed,
    /// Session failed with an error.
    Failed,
    /// Session was cancelled.
    Cancelled,
}

impl std::fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionStatus::Pending => write!(f, "pending"),
            SessionStatus::Running => write!(f, "running"),
            SessionStatus::Completed => write!(f, "completed"),
            SessionStatus::Failed => write!(f, "failed"),
            SessionStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// Agent session record.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Session {
    /// Unique session identifier.
    pub id: Uuid,
    /// Agent identifier.
    pub agent_id: String,
    /// Workspace path for the session.
    pub workspace_path: String,
    /// Prompt or task description.
    pub prompt: String,
    /// Current session status.
    pub status: SessionStatus,
    /// Result output (if completed).
    pub result: Option<String>,
    /// Error message (if failed).
    pub error: Option<String>,
    /// When the session was created.
    pub created_at: OffsetDateTime,
    /// When the session started running.
    pub started_at: Option<OffsetDateTime>,
    /// When the session finished.
    pub finished_at: Option<OffsetDateTime>,
    /// Priority level (higher = more important).
    pub priority: i32,
}

/// Parameters for creating a new session.
#[derive(Debug, Clone)]
pub struct CreateSessionParams {
    /// Agent identifier.
    pub agent_id: String,
    /// Workspace path.
    pub workspace_path: String,
    /// Prompt or task.
    pub prompt: String,
    /// Priority level.
    pub priority: i32,
}

/// Detailed session status information.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum SessionStatusDetail {
    /// Session is pending and waiting to be started.
    Pending {
        /// Position in the queue (0-indexed).
        queue_position: usize,
    },
    /// Session is currently running.
    Running {
        /// When the session started.
        started_at: OffsetDateTime,
    },
    /// Session completed successfully.
    Completed {
        /// Result output.
        result: String,
        /// When the session finished.
        finished_at: OffsetDateTime,
    },
    /// Session failed with an error.
    Failed {
        /// Error message.
        error: String,
        /// When the session finished.
        finished_at: OffsetDateTime,
    },
}

impl From<Session> for SessionStatusDetail {
    fn from(session: Session) -> Self {
        match session.status {
            SessionStatus::Pending => SessionStatusDetail::Pending { queue_position: 0 },
            SessionStatus::Running => SessionStatusDetail::Running {
                started_at: session.started_at.unwrap_or_else(OffsetDateTime::now_utc),
            },
            SessionStatus::Completed => SessionStatusDetail::Completed {
                result: session.result.unwrap_or_default(),
                finished_at: session.finished_at.unwrap_or_else(OffsetDateTime::now_utc),
            },
            SessionStatus::Failed => SessionStatusDetail::Failed {
                error: session.error.unwrap_or_default(),
                finished_at: session.finished_at.unwrap_or_else(OffsetDateTime::now_utc),
            },
            SessionStatus::Cancelled => SessionStatusDetail::Failed {
                error: "Session was cancelled".to_string(),
                finished_at: session.finished_at.unwrap_or_else(OffsetDateTime::now_utc),
            },
        }
    }
}

//! Session management for agent operations.
//!
//! This module provides functionality for creating, tracking, and managing
//! agent sessions.

pub use handle::SessionHandle;

mod handle;

use crate::{
    Result,
    db::Database,
    models::{CreateSessionParams, Session, SessionStatus},
};
use time::OffsetDateTime;
use uuid::Uuid;

/// Manager for agent sessions.
#[derive(Debug, Clone)]
pub struct SessionManager {
    db: Database,
}

impl SessionManager {
    /// Create a new session manager.
    pub async fn new(database_url: &str) -> Result<Self> {
        let db = Database::new(database_url).await?;
        Ok(Self { db })
    }

    /// Create a new session.
    pub async fn create(&self, params: CreateSessionParams) -> Result<Session> {
        let id = Uuid::new_v4();
        let now = OffsetDateTime::now_utc();

        sqlx::query_as::<_, Session>(
            r#"
            INSERT INTO sessions (id, agent_id, workspace_path, prompt, status, created_at, priority)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            RETURNING *
            "#
        )
        .bind(id.as_bytes().as_slice())
        .bind(&params.agent_id)
        .bind(&params.workspace_path)
        .bind(&params.prompt)
        .bind(SessionStatus::Pending)
        .bind(now)
        .bind(params.priority)
        .fetch_one(self.db.pool())
        .await
        .map_err(Into::into)
    }

    /// Get a session by ID.
    pub async fn get(&self, id: Uuid) -> Result<Option<Session>> {
        sqlx::query_as::<_, Session>("SELECT * FROM sessions WHERE id = ?1")
            .bind(id.as_bytes().as_slice())
            .fetch_optional(self.db.pool())
            .await
            .map_err(Into::into)
    }

    /// List all sessions ordered by creation time.
    pub async fn list(&self) -> Result<Vec<Session>> {
        sqlx::query_as::<_, Session>("SELECT * FROM sessions ORDER BY created_at DESC")
            .fetch_all(self.db.pool())
            .await
            .map_err(Into::into)
    }

    /// List pending sessions ordered by priority and creation time.
    pub async fn list_pending(&self) -> Result<Vec<Session>> {
        sqlx::query_as::<_, Session>(
            "SELECT * FROM sessions WHERE status = 'pending' ORDER BY priority DESC, created_at ASC"
        )
        .fetch_all(self.db.pool())
        .await
        .map_err(Into::into)
    }

    /// Start a pending session.
    pub async fn start(&self, id: Uuid) -> Result<Session> {
        let now = OffsetDateTime::now_utc();

        sqlx::query_as::<_, Session>(
            r#"
            UPDATE sessions
            SET status = 'running', started_at = ?1
            WHERE id = ?2 AND status = 'pending'
            RETURNING *
            "#,
        )
        .bind(now)
        .bind(id.as_bytes().as_slice())
        .fetch_optional(self.db.pool())
        .await?
        .ok_or_else(|| crate::Error::SessionNotFound(id))
    }

    /// Complete a running session with a result.
    pub async fn complete(&self, id: Uuid, result: String) -> Result<Session> {
        let now = OffsetDateTime::now_utc();

        sqlx::query_as::<_, Session>(
            r#"
            UPDATE sessions
            SET status = 'completed', result = ?1, finished_at = ?2
            WHERE id = ?3 AND status = 'running'
            RETURNING *
            "#,
        )
        .bind(result)
        .bind(now)
        .bind(id.as_bytes().as_slice())
        .fetch_optional(self.db.pool())
        .await?
        .ok_or_else(|| crate::Error::SessionNotFound(id))
    }

    /// Fail a running session with an error message.
    pub async fn fail(&self, id: Uuid, error: String) -> Result<Session> {
        let now = OffsetDateTime::now_utc();

        sqlx::query_as::<_, Session>(
            r#"
            UPDATE sessions
            SET status = 'failed', error = ?1, finished_at = ?2
            WHERE id = ?3 AND status = 'running'
            RETURNING *
            "#,
        )
        .bind(error)
        .bind(now)
        .bind(id.as_bytes().as_slice())
        .fetch_optional(self.db.pool())
        .await?
        .ok_or_else(|| crate::Error::SessionNotFound(id))
    }

    /// Cancel a pending session.
    pub async fn cancel(&self, id: Uuid) -> Result<Session> {
        let now = OffsetDateTime::now_utc();

        sqlx::query_as::<_, Session>(
            r#"
            UPDATE sessions
            SET status = 'cancelled', finished_at = ?1
            WHERE id = ?2 AND status = 'pending'
            RETURNING *
            "#,
        )
        .bind(now)
        .bind(id.as_bytes().as_slice())
        .fetch_optional(self.db.pool())
        .await?
        .ok_or_else(|| crate::Error::SessionNotFound(id))
    }
}

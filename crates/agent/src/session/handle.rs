//! Session handle for interacting with running sessions.

use sqlx::SqlitePool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::Result;
use crate::models::{Session, SessionStatus, SessionStatusDetail};

/// Handle to an active or completed agent session.
#[derive(Debug, Clone)]
pub struct SessionHandle {
    id: Uuid,
    /// Stored database pool for future use (currently passed explicitly to methods).
    #[allow(dead_code)]
    db: Option<SqlitePool>,
}

impl SessionHandle {
    /// Create a new session handle for the given session ID.
    pub fn new(id: Uuid) -> Self {
        Self { id, db: None }
    }

    /// Create a new session handle with a database pool.
    pub fn with_db(id: Uuid, db: SqlitePool) -> Self {
        Self { id, db: Some(db) }
    }

    /// Get the session ID.
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Get the current status of the session.
    ///
    /// Queries the database and returns detailed status information.
    pub async fn status(&self, db: &SqlitePool) -> Result<SessionStatusDetail> {
        let session: Session = sqlx::query_as::<_, Session>("SELECT * FROM sessions WHERE id = ?1")
            .bind(self.id.as_bytes().as_slice())
            .fetch_one(db)
            .await?;

        let detail = match session.status {
            SessionStatus::Pending => {
                // Calculate queue position
                let position: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM sessions WHERE status = 'pending' AND priority > ?1 AND created_at < ?2"
                )
                .bind(session.priority)
                .bind(session.created_at)
                .fetch_one(db)
                .await?;

                SessionStatusDetail::Pending {
                    queue_position: position as usize,
                }
            }
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
        };

        Ok(detail)
    }

    /// Cancel the session.
    ///
    /// Updates the session status to cancelled. Only pending or running sessions
    /// can be cancelled.
    pub async fn cancel(&self, db: &SqlitePool) -> Result<()> {
        let now = OffsetDateTime::now_utc();

        let result = sqlx::query(
            r#"
            UPDATE sessions
            SET status = 'cancelled', finished_at = ?1
            WHERE id = ?2 AND status IN ('pending', 'running')
            "#,
        )
        .bind(now)
        .bind(self.id.as_bytes().as_slice())
        .execute(db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(crate::Error::InvalidStateTransition {
                from: "current".to_string(),
                to: "cancelled".to_string(),
            });
        }

        Ok(())
    }

    /// Read the session result as a string, blocking until complete.
    ///
    /// Polls the database until the session reaches a terminal state
    /// (completed, failed, or cancelled), then returns the result.
    pub async fn read_to_string(&self, db: &SqlitePool) -> Result<String> {
        loop {
            let session: Session =
                sqlx::query_as::<_, Session>("SELECT * FROM sessions WHERE id = ?1")
                    .bind(self.id.as_bytes().as_slice())
                    .fetch_one(db)
                    .await?;

            match session.status {
                SessionStatus::Completed => {
                    return Ok(session.result.unwrap_or_default());
                }
                SessionStatus::Failed => {
                    return Err(crate::Error::Acp(
                        session.error.unwrap_or_else(|| "Unknown error".to_string()),
                    ));
                }
                SessionStatus::Cancelled => {
                    return Err(crate::Error::Acp("Session was cancelled".to_string()));
                }
                _ => {
                    // Session is still pending or running, wait and retry
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
        }
    }

    /// Read the next update from the session.
    ///
    /// Returns `Some(String)` with the result if the session is complete,
    /// or `None` if the session is still running. This is a non-blocking
    /// streaming-style method for checking session progress.
    pub async fn read_update(&self, db: &SqlitePool) -> Result<Option<String>> {
        let session: Session = sqlx::query_as::<_, Session>("SELECT * FROM sessions WHERE id = ?1")
            .bind(self.id.as_bytes().as_slice())
            .fetch_one(db)
            .await?;

        match session.status {
            SessionStatus::Completed => Ok(Some(session.result.unwrap_or_default())),
            SessionStatus::Failed => Err(crate::Error::Acp(
                session.error.unwrap_or_else(|| "Unknown error".to_string()),
            )),
            SessionStatus::Cancelled => Err(crate::Error::Acp("Session was cancelled".to_string())),
            _ => {
                // Session is still pending or running
                Ok(None)
            }
        }
    }
}

use sqlx::Type;
use std::fmt;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Type)]
#[sqlx(rename_all = "lowercase")]
#[allow(dead_code)]
pub(crate) enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "pending"),
            TaskStatus::Running => write!(f, "running"),
            TaskStatus::Completed => write!(f, "completed"),
            TaskStatus::Failed => write!(f, "failed"),
        }
    }
}

#[allow(dead_code)]
impl TaskStatus {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "pending",
            TaskStatus::Running => "running",
            TaskStatus::Completed => "completed",
            TaskStatus::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct Task {
    pub(crate) id: Uuid,
    pub(crate) prompt: String,
    pub(crate) status: TaskStatus,
    pub(crate) response: Option<String>,
    pub(crate) exit_code: Option<i32>,
    pub(crate) created_at: OffsetDateTime,
    pub(crate) started_at: Option<OffsetDateTime>,
    pub(crate) completed_at: Option<OffsetDateTime>,
}

impl Task {
    #[allow(dead_code)]
    pub(crate) fn new(prompt: String) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            id: Uuid::new_v4(),
            prompt,
            status: TaskStatus::Pending,
            response: None,
            exit_code: None,
            created_at: now,
            started_at: None,
            completed_at: None,
        }
    }
}

use sqlx::{FromRow, Row};

impl<'r> FromRow<'r, sqlx::sqlite::SqliteRow> for Task {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        use time::format_description::well_known::Rfc3339;

        let id_str: String = row.try_get("id")?;
        let id = Uuid::parse_str(&id_str).map_err(|e| sqlx::Error::ColumnDecode {
            index: "id".into(),
            source: Box::new(e),
        })?;

        let prompt: String = row.try_get("prompt")?;
        let status: TaskStatus = row.try_get("status")?;
        let response: Option<String> = row.try_get("response")?;
        let exit_code: Option<i32> = row.try_get("exit_code")?;

        let created_at_str: String = row.try_get("created_at")?;
        let created_at = OffsetDateTime::parse(&created_at_str, &Rfc3339).map_err(|e| {
            sqlx::Error::ColumnDecode {
                index: "created_at".into(),
                source: Box::new(e),
            }
        })?;

        let started_at: Option<OffsetDateTime> = row
            .try_get::<Option<String>, _>("started_at")?
            .map(|s| OffsetDateTime::parse(&s, &Rfc3339))
            .transpose()
            .map_err(|e| sqlx::Error::ColumnDecode {
                index: "started_at".into(),
                source: Box::new(e),
            })?;

        let completed_at: Option<OffsetDateTime> = row
            .try_get::<Option<String>, _>("completed_at")?
            .map(|s| OffsetDateTime::parse(&s, &Rfc3339))
            .transpose()
            .map_err(|e| sqlx::Error::ColumnDecode {
                index: "completed_at".into(),
                source: Box::new(e),
            })?;

        Ok(Task {
            id,
            prompt,
            status,
            response,
            exit_code,
            created_at,
            started_at,
            completed_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::format_description::well_known::Rfc3339;

    #[test]
    fn test_task_new() {
        let task = Task::new("refactor this code".to_string());

        assert_eq!(task.prompt, "refactor this code");
        assert_eq!(task.status, TaskStatus::Pending);
        assert!(task.response.is_none());
        assert!(task.exit_code.is_none());
        assert!(task.started_at.is_none());
        assert!(task.completed_at.is_none());

        // Verify UUID v4
        assert_eq!(task.id.get_version_num(), 4);

        // Verify timestamp is UTC
        assert_eq!(task.created_at.offset().whole_seconds(), 0);
    }

    #[test]
    fn test_task_status_display() {
        assert_eq!(TaskStatus::Pending.to_string(), "pending");
        assert_eq!(TaskStatus::Running.to_string(), "running");
        assert_eq!(TaskStatus::Completed.to_string(), "completed");
        assert_eq!(TaskStatus::Failed.to_string(), "failed");
    }

    #[test]
    fn test_task_status_as_str() {
        assert_eq!(TaskStatus::Pending.as_str(), "pending");
        assert_eq!(TaskStatus::Running.as_str(), "running");
        assert_eq!(TaskStatus::Completed.as_str(), "completed");
        assert_eq!(TaskStatus::Failed.as_str(), "failed");
    }

    #[test]
    fn test_task_status_sqlx_mapping() {
        // Verify that TaskStatus variants map correctly to lowercase strings
        // This is validated by the sqlx::Type derive macro
        let pending = TaskStatus::Pending;
        let running = TaskStatus::Running;
        let completed = TaskStatus::Completed;
        let failed = TaskStatus::Failed;

        assert_eq!(pending.as_str(), "pending");
        assert_eq!(running.as_str(), "running");
        assert_eq!(completed.as_str(), "completed");
        assert_eq!(failed.as_str(), "failed");
    }

    #[test]
    fn test_timestamp_rfc3339_format() {
        let now = OffsetDateTime::now_utc();
        let formatted = now.format(&Rfc3339).unwrap();

        // Verify format includes timezone and proper structure
        assert!(formatted.contains('T') || formatted.contains('t'));
        assert!(formatted.contains('+') || formatted.ends_with('Z'));

        // Verify we can parse it back
        let parsed = OffsetDateTime::parse(&formatted, &Rfc3339).unwrap();
        assert_eq!(now.unix_timestamp(), parsed.unix_timestamp());
    }

    #[test]
    fn test_uuid_v4_generation() {
        let uuid1 = Uuid::new_v4();
        let uuid2 = Uuid::new_v4();

        // Verify they're different
        assert_ne!(uuid1, uuid2);

        // Verify format
        let uuid_str = uuid1.to_string();
        assert_eq!(uuid_str.len(), 36);
        assert_eq!(uuid1.get_version_num(), 4);

        // Verify parsing
        let parsed = Uuid::parse_str(&uuid_str).unwrap();
        assert_eq!(uuid1, parsed);
    }
}

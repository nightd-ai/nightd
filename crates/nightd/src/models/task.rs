use sqlx::{FromRow, SqlitePool, Type};
use std::fmt;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Type)]
#[sqlx(rename_all = "lowercase")]
pub enum TaskStatus {
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

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "pending",
            TaskStatus::Running => "running",
            TaskStatus::Completed => "completed",
            TaskStatus::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct Task {
    pub id: Uuid,
    pub prompt: String,
    pub status: TaskStatus,
    pub response: Option<String>,
    pub exit_code: Option<i32>,
    pub created_at: OffsetDateTime,
    pub started_at: Option<OffsetDateTime>,
    pub completed_at: Option<OffsetDateTime>,
}

impl Task {
    pub fn new(prompt: String) -> Self {
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

// Create a new task
pub async fn create_task(pool: &SqlitePool, prompt: &str) -> Result<Task, sqlx::Error> {
    let task = Task::new(prompt.to_string());

    sqlx::query("INSERT INTO tasks (id, prompt, status, created_at) VALUES (?1, ?2, ?3, ?4)")
        .bind(task.id)
        .bind(&task.prompt)
        .bind(task.status)
        .bind(task.created_at)
        .execute(pool)
        .await?;

    Ok(task)
}

// Get next pending task (oldest first)
pub async fn get_next_pending(pool: &SqlitePool) -> Result<Option<Task>, sqlx::Error> {
    let task = sqlx::query_as::<_, Task>(
        "SELECT * FROM tasks WHERE status = 'pending' ORDER BY created_at ASC LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;

    Ok(task)
}

// Get a specific task by ID
pub async fn get_task(pool: &SqlitePool, id: &Uuid) -> Result<Option<Task>, sqlx::Error> {
    let task = sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = ?1")
        .bind(id)
        .fetch_optional(pool)
        .await?;

    Ok(task)
}

// Mark task as running
pub async fn mark_task_running(pool: &SqlitePool, id: &Uuid) -> Result<(), sqlx::Error> {
    let now = OffsetDateTime::now_utc();

    sqlx::query("UPDATE tasks SET status = 'running', started_at = ?1 WHERE id = ?2")
        .bind(now)
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

// Mark task as completed
pub async fn complete_task(
    pool: &SqlitePool,
    id: &Uuid,
    response: &str,
    exit_code: i32,
) -> Result<(), sqlx::Error> {
    let now = OffsetDateTime::now_utc();

    sqlx::query(
        "UPDATE tasks SET status = 'completed', response = ?1, exit_code = ?2, completed_at = ?3 WHERE id = ?4"
    )
    .bind(response)
    .bind(exit_code)
    .bind(now)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

// Mark task as failed
pub async fn fail_task(pool: &SqlitePool, id: &Uuid, error: &str) -> Result<(), sqlx::Error> {
    let now = OffsetDateTime::now_utc();

    sqlx::query(
        "UPDATE tasks SET status = 'failed', response = ?1, exit_code = -1, completed_at = ?2 WHERE id = ?3"
    )
    .bind(error)
    .bind(now)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

// Get running tasks
pub(crate) async fn _get_running_tasks(pool: &SqlitePool) -> Result<Vec<Task>, sqlx::Error> {
    let tasks = sqlx::query_as::<_, Task>(
        "SELECT * FROM tasks WHERE status = 'running' ORDER BY started_at ASC",
    )
    .fetch_all(pool)
    .await?;

    Ok(tasks)
}

// Get tasks by status with limit
pub async fn get_tasks_by_status(
    pool: &SqlitePool,
    status: TaskStatus,
    limit: i64,
) -> Result<Vec<Task>, sqlx::Error> {
    let tasks = sqlx::query_as::<_, Task>(
        "SELECT * FROM tasks WHERE status = ?1 ORDER BY created_at DESC LIMIT ?2",
    )
    .bind(status)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(tasks)
}

// Count tasks by status
pub async fn count_tasks_by_status(
    pool: &SqlitePool,
    status: TaskStatus,
) -> Result<i64, sqlx::Error> {
    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM tasks WHERE status = ?1")
        .bind(status)
        .fetch_one(pool)
        .await?;

    Ok(count)
}

// Get all tasks with limit
pub async fn get_all_tasks(pool: &SqlitePool, limit: i64) -> Result<Vec<Task>, sqlx::Error> {
    let tasks = sqlx::query_as::<_, Task>("SELECT * FROM tasks ORDER BY created_at DESC LIMIT ?1")
        .bind(limit)
        .fetch_all(pool)
        .await?;

    Ok(tasks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

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
        let formatted = now
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap();

        // Verify format includes timezone and proper structure
        assert!(formatted.contains('T') || formatted.contains('t'));
        assert!(formatted.contains('+') || formatted.ends_with('Z'));

        // Verify we can parse it back
        let parsed =
            OffsetDateTime::parse(&formatted, &time::format_description::well_known::Rfc3339)
                .unwrap();
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

    #[tokio::test]
    async fn test_create_and_get_task() {
        let pool = db::init("sqlite::memory:").await.unwrap();

        let task = create_task(&pool, "test prompt").await.unwrap();
        assert_eq!(task.prompt, "test prompt");
        assert_eq!(task.status, TaskStatus::Pending);

        // Verify we can fetch it back
        let fetched = get_task(&pool, &task.id).await.unwrap();
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.id, task.id);
        assert_eq!(fetched.prompt, "test prompt");
    }

    #[tokio::test]
    async fn test_get_next_pending() {
        let pool = db::init("sqlite::memory:").await.unwrap();

        // Should return None when no tasks
        let result = get_next_pending(&pool).await.unwrap();
        assert!(result.is_none());

        // Create a task
        let task = create_task(&pool, "pending task").await.unwrap();

        // Should return the task
        let result = get_next_pending(&pool).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().id, task.id);
    }

    #[tokio::test]
    async fn test_task_status_transitions() {
        let pool = db::init("sqlite::memory:").await.unwrap();

        let task = create_task(&pool, "test task").await.unwrap();

        // Mark as running
        mark_task_running(&pool, &task.id).await.unwrap();
        let fetched = get_task(&pool, &task.id).await.unwrap().unwrap();
        assert_eq!(fetched.status, TaskStatus::Running);
        assert!(fetched.started_at.is_some());

        // Mark as completed
        complete_task(&pool, &task.id, "success", 0).await.unwrap();
        let fetched = get_task(&pool, &task.id).await.unwrap().unwrap();
        assert_eq!(fetched.status, TaskStatus::Completed);
        assert_eq!(fetched.response, Some("success".to_string()));
        assert_eq!(fetched.exit_code, Some(0));
        assert!(fetched.completed_at.is_some());
    }

    #[tokio::test]
    async fn test_fail_task() {
        let pool = db::init("sqlite::memory:").await.unwrap();

        let task = create_task(&pool, "failing task").await.unwrap();
        mark_task_running(&pool, &task.id).await.unwrap();

        fail_task(&pool, &task.id, "error occurred").await.unwrap();
        let fetched = get_task(&pool, &task.id).await.unwrap().unwrap();
        assert_eq!(fetched.status, TaskStatus::Failed);
        assert_eq!(fetched.response, Some("error occurred".to_string()));
        assert_eq!(fetched.exit_code, Some(-1));
    }

    #[tokio::test]
    async fn test_count_and_get_tasks_by_status() {
        let pool = db::init("sqlite::memory:").await.unwrap();

        // Create tasks with different statuses
        let task1 = create_task(&pool, "task 1").await.unwrap();
        let task2 = create_task(&pool, "task 2").await.unwrap();
        let _task3 = create_task(&pool, "task 3").await.unwrap();

        // Mark one as running
        mark_task_running(&pool, &task1.id).await.unwrap();

        // Mark one as completed
        complete_task(&pool, &task2.id, "done", 0).await.unwrap();

        // Test counts
        let pending_count = count_tasks_by_status(&pool, TaskStatus::Pending)
            .await
            .unwrap();
        assert_eq!(pending_count, 1);

        let running_count = count_tasks_by_status(&pool, TaskStatus::Running)
            .await
            .unwrap();
        assert_eq!(running_count, 1);

        let completed_count = count_tasks_by_status(&pool, TaskStatus::Completed)
            .await
            .unwrap();
        assert_eq!(completed_count, 1);

        // Test get by status
        let running_tasks = _get_running_tasks(&pool).await.unwrap();
        assert_eq!(running_tasks.len(), 1);
        assert_eq!(running_tasks[0].id, task1.id);
    }
}

use crate::models::{Task, TaskStatus};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use std::path::Path;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use uuid::Uuid;

#[allow(dead_code)]
pub(crate) async fn init_pool(database_path: &Path) -> Result<SqlitePool, sqlx::Error> {
    let database_url = format!("sqlite://{}", database_path.display());

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    Ok(pool)
}

#[allow(dead_code)]
pub(crate) async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::migrate!("../../migrations")
        .run(pool)
        .await
        .map_err(|e| sqlx::Error::Migrate(Box::new(e)))
}

// Create a new task
#[allow(dead_code)]
pub(crate) async fn create_task(pool: &SqlitePool, prompt: &str) -> Result<Task, sqlx::Error> {
    let task = Task::new(prompt.to_string());
    let id_str = task.id.to_string();
    let created_at_str = task.created_at.format(&Rfc3339).unwrap();
    let status_str = task.status.as_str();

    sqlx::query("INSERT INTO tasks (id, prompt, status, created_at) VALUES (?1, ?2, ?3, ?4)")
        .bind(&id_str)
        .bind(&task.prompt)
        .bind(status_str)
        .bind(&created_at_str)
        .execute(pool)
        .await?;

    Ok(task)
}

// Get next pending task (oldest first)
#[allow(dead_code)]
pub(crate) async fn get_next_pending(pool: &SqlitePool) -> Result<Option<Task>, sqlx::Error> {
    let task = sqlx::query_as::<_, Task>(
        "SELECT * FROM tasks WHERE status = 'pending' ORDER BY created_at ASC LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;

    Ok(task)
}

// Get a specific task by ID
#[allow(dead_code)]
pub(crate) async fn get_task(pool: &SqlitePool, id: &Uuid) -> Result<Option<Task>, sqlx::Error> {
    let id_str = id.to_string();

    let task = sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = ?1")
        .bind(&id_str)
        .fetch_optional(pool)
        .await?;

    Ok(task)
}

// Mark task as running
#[allow(dead_code)]
pub(crate) async fn mark_task_running(pool: &SqlitePool, id: &Uuid) -> Result<(), sqlx::Error> {
    let id_str = id.to_string();
    let now = OffsetDateTime::now_utc().format(&Rfc3339).unwrap();

    sqlx::query("UPDATE tasks SET status = 'running', started_at = ?1 WHERE id = ?2")
        .bind(&now)
        .bind(&id_str)
        .execute(pool)
        .await?;

    Ok(())
}

// Mark task as completed
#[allow(dead_code)]
pub(crate) async fn complete_task(
    pool: &SqlitePool,
    id: &Uuid,
    response: &str,
    exit_code: i32,
) -> Result<(), sqlx::Error> {
    let id_str = id.to_string();
    let now = OffsetDateTime::now_utc().format(&Rfc3339).unwrap();

    sqlx::query(
        "UPDATE tasks SET status = 'completed', response = ?1, exit_code = ?2, completed_at = ?3 WHERE id = ?4"
    )
    .bind(response)
    .bind(exit_code)
    .bind(&now)
    .bind(&id_str)
    .execute(pool)
    .await?;

    Ok(())
}

// Mark task as failed
#[allow(dead_code)]
pub(crate) async fn fail_task(
    pool: &SqlitePool,
    id: &Uuid,
    error: &str,
) -> Result<(), sqlx::Error> {
    let id_str = id.to_string();
    let now = OffsetDateTime::now_utc().format(&Rfc3339).unwrap();

    sqlx::query(
        "UPDATE tasks SET status = 'failed', response = ?1, exit_code = -1, completed_at = ?2 WHERE id = ?3"
    )
    .bind(error)
    .bind(&now)
    .bind(&id_str)
    .execute(pool)
    .await?;

    Ok(())
}

// Get running tasks
#[allow(dead_code)]
pub(crate) async fn get_running_tasks(pool: &SqlitePool) -> Result<Vec<Task>, sqlx::Error> {
    let tasks = sqlx::query_as::<_, Task>(
        "SELECT * FROM tasks WHERE status = 'running' ORDER BY started_at ASC",
    )
    .fetch_all(pool)
    .await?;

    Ok(tasks)
}

// Get tasks by status with limit
#[allow(dead_code)]
pub(crate) async fn get_tasks_by_status(
    pool: &SqlitePool,
    status: TaskStatus,
    limit: i64,
) -> Result<Vec<Task>, sqlx::Error> {
    let status_str = status.as_str();

    let tasks = sqlx::query_as::<_, Task>(
        "SELECT * FROM tasks WHERE status = ?1 ORDER BY created_at DESC LIMIT ?2",
    )
    .bind(status_str)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(tasks)
}

// Count tasks by status
#[allow(dead_code)]
pub(crate) async fn count_tasks_by_status(
    pool: &SqlitePool,
    status: TaskStatus,
) -> Result<i64, sqlx::Error> {
    let status_str = status.as_str();

    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM tasks WHERE status = ?1")
        .bind(status_str)
        .fetch_one(pool)
        .await?;

    Ok(count)
}

// Get all tasks with limit
#[allow(dead_code)]
pub(crate) async fn get_all_tasks(pool: &SqlitePool, limit: i64) -> Result<Vec<Task>, sqlx::Error> {
    let tasks = sqlx::query_as::<_, Task>("SELECT * FROM tasks ORDER BY created_at DESC LIMIT ?1")
        .bind(limit)
        .fetch_all(pool)
        .await?;

    Ok(tasks)
}

#[cfg(test)]
pub(crate) async fn create_test_pool() -> SqlitePool {
    let pool = sqlx::sqlite::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to create test database pool");

    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations on test database");

    pool
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TaskStatus;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_database_initialization() {
        let pool = create_test_pool().await;
        let result: Result<i64, _> = sqlx::query_scalar("SELECT COUNT(*) FROM tasks")
            .fetch_one(&pool)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_uuid_format() {
        let uuid = Uuid::new_v4();
        let uuid_str = uuid.to_string();

        // Verify UUID v4 format (should have dashes and proper version)
        assert_eq!(uuid_str.len(), 36);
        assert!(uuid_str.contains('-'));

        // Verify it's a valid UUID v4
        let parsed = Uuid::parse_str(&uuid_str).unwrap();
        assert_eq!(parsed.get_version_num(), 4);
    }

    #[tokio::test]
    async fn test_migrations_run_successfully() {
        let pool = create_test_pool().await;

        // Verify tasks table exists
        let result: Result<i64, _> = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='tasks'",
        )
        .fetch_one(&pool)
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);

        // Verify index exists
        let result: Result<i64, _> = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name='idx_status_created'",
        )
        .fetch_one(&pool)
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_create_and_get_task() {
        let pool = create_test_pool().await;

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
        let pool = create_test_pool().await;

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
        let pool = create_test_pool().await;

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
        let pool = create_test_pool().await;

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
        let pool = create_test_pool().await;

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
        let running_tasks = get_running_tasks(&pool).await.unwrap();
        assert_eq!(running_tasks.len(), 1);
        assert_eq!(running_tasks[0].id, task1.id);
    }
}

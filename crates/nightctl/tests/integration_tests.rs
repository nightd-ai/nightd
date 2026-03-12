use nightd::db;
use nightd::models::{
    TaskStatus, count_tasks_by_status, create_task, fail_task, get_all_tasks, mark_task_running,
};
use std::path::PathBuf;

#[tokio::test]
async fn test_run_command_creates_task() {
    let pool = db::init(PathBuf::from(":memory:")).await.unwrap();

    // Create a task via the API to verify it works
    let task = create_task(&pool, "test prompt").await.unwrap();
    assert_eq!(task.prompt, "test prompt");
    assert_eq!(task.status, TaskStatus::Pending);
}

#[tokio::test]
async fn test_status_shows_counts() {
    let pool = db::init(PathBuf::from(":memory:")).await.unwrap();

    // Create tasks in different states
    let running = create_task(&pool, "running task").await.unwrap();
    mark_task_running(&pool, &running.id).await.unwrap();

    let _pending = create_task(&pool, "pending task").await.unwrap();

    let failed = create_task(&pool, "failed task").await.unwrap();
    fail_task(&pool, &failed.id, "error").await.unwrap();

    // Verify counts
    let running_count = count_tasks_by_status(&pool, TaskStatus::Running)
        .await
        .unwrap();
    let pending_count = count_tasks_by_status(&pool, TaskStatus::Pending)
        .await
        .unwrap();
    let failed_count = count_tasks_by_status(&pool, TaskStatus::Failed)
        .await
        .unwrap();

    assert_eq!(running_count, 1);
    assert_eq!(pending_count, 1);
    assert_eq!(failed_count, 1);
}

#[tokio::test]
async fn test_list_tasks_format() {
    let pool = db::init(PathBuf::from(":memory:")).await.unwrap();

    // Create tasks
    create_task(&pool, "task 1").await.unwrap();
    create_task(&pool, "task 2").await.unwrap();

    // Verify we can fetch them
    let tasks = get_all_tasks(&pool, 10).await.unwrap();
    assert_eq!(tasks.len(), 2);
}

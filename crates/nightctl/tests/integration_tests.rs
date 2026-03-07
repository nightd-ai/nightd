use nightd::api::create_app;
use nightd::db;
use nightd::models::TaskStatus;
use sqlx::SqlitePool;
use tokio::time::{Duration, sleep};

async fn create_test_pool() -> SqlitePool {
    let pool = sqlx::sqlite::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to create test database pool");

    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations on test database");

    pool
}

async fn start_test_daemon(pool: SqlitePool) -> u16 {
    let port = 0; // Let OS assign port
    let app = create_app(pool);
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .expect("Failed to bind to port");
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    sleep(Duration::from_millis(100)).await;
    port
}

#[tokio::test]
async fn test_run_command_creates_task() {
    let pool = create_test_pool().await;

    // Create a task via the API to verify it works
    let task = db::create_task(&pool, "test prompt").await.unwrap();
    assert_eq!(task.prompt, "test prompt");
    assert_eq!(task.status, TaskStatus::Pending);
}

#[tokio::test]
async fn test_status_shows_counts() {
    let pool = create_test_pool().await;

    // Create tasks in different states
    let running = db::create_task(&pool, "running task").await.unwrap();
    db::mark_task_running(&pool, &running.id).await.unwrap();

    let _pending = db::create_task(&pool, "pending task").await.unwrap();

    let failed = db::create_task(&pool, "failed task").await.unwrap();
    db::fail_task(&pool, &failed.id, "error").await.unwrap();

    // Verify counts
    let running_count = db::count_tasks_by_status(&pool, TaskStatus::Running)
        .await
        .unwrap();
    let pending_count = db::count_tasks_by_status(&pool, TaskStatus::Pending)
        .await
        .unwrap();
    let failed_count = db::count_tasks_by_status(&pool, TaskStatus::Failed)
        .await
        .unwrap();

    assert_eq!(running_count, 1);
    assert_eq!(pending_count, 1);
    assert_eq!(failed_count, 1);
}

#[tokio::test]
async fn test_list_tasks_format() {
    let pool = create_test_pool().await;

    // Create tasks
    db::create_task(&pool, "task 1").await.unwrap();
    db::create_task(&pool, "task 2").await.unwrap();

    // Verify we can fetch them
    let tasks = db::get_all_tasks(&pool, 10).await.unwrap();
    assert_eq!(tasks.len(), 2);
}

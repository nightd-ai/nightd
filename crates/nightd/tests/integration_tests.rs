use axum::body::Body;
use axum::http::{Request, StatusCode};
use nightd::api::create_app;
use nightd::db;
use nightd::models::{
    TaskStatus, complete_task, create_task, fail_task, get_all_tasks, get_task, mark_task_running,
};
use tower::util::ServiceExt;

#[tokio::test]
async fn test_status_endpoint() {
    let pool = db::init("sqlite::memory:").await.unwrap();
    let app = create_app(pool);

    let response: axum::response::Response = app
        .oneshot(
            Request::builder()
                .uri("/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_status_response_body() {
    let pool = db::init("sqlite::memory:").await.unwrap();
    let app = create_app(pool);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "OK");
    assert!(json["running_tasks"].is_number());
    assert!(json["pending_tasks"].is_number());
    assert!(json["failed_tasks"].is_number());
}

#[tokio::test]
async fn test_create_task_endpoint() {
    let pool = db::init("sqlite::memory:").await.unwrap();
    let app = create_app(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/tasks")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"prompt": "refactor this code"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["task_id"].as_str().unwrap().len() > 0);
    assert_eq!(json["status"], "pending");
}

#[tokio::test]
async fn test_create_task_invalid_json() {
    let pool = db::init("sqlite::memory:").await.unwrap();
    let app = create_app(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/tasks")
                .header("content-type", "application/json")
                .body(Body::from(r#"invalid json"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_list_tasks_endpoint() {
    let pool = db::init("sqlite::memory:").await.unwrap();

    // Create some tasks
    create_task(&pool, "task 1").await.unwrap();
    create_task(&pool, "task 2").await.unwrap();

    let app = create_app(pool);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/tasks")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["tasks"].is_array());
    assert_eq!(json["total"], 2);
    assert_eq!(json["tasks"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_list_tasks_with_status_filter() {
    let pool = db::init("sqlite::memory:").await.unwrap();

    // Create tasks with different statuses
    let task = create_task(&pool, "pending task").await.unwrap();
    complete_task(&pool, &task.id, "done", 0).await.unwrap();
    create_task(&pool, "another pending").await.unwrap();

    let app = create_app(pool);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/tasks?status=pending")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["total"], 1);
    assert_eq!(json["tasks"][0]["status"], "pending");
}

#[tokio::test]
async fn test_list_tasks_with_limit() {
    let pool = db::init("sqlite::memory:").await.unwrap();

    // Create many tasks
    for i in 0..10 {
        create_task(&pool, &format!("task {}", i)).await.unwrap();
    }

    let app = create_app(pool);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/tasks?limit=3")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["total"], 3);
}

#[tokio::test]
async fn test_get_task_endpoint() {
    let pool = db::init("sqlite::memory:").await.unwrap();

    let task = create_task(&pool, "test task").await.unwrap();
    let task_id = task.id.to_string();

    let app = create_app(pool);

    let response = app
        .oneshot(
            Request::builder()
                .uri(&format!("/tasks/{}", task_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["id"], task_id);
    assert_eq!(json["prompt"], "test task");
    assert_eq!(json["status"], "pending");
}

#[tokio::test]
async fn test_get_task_not_found() {
    let pool = db::init("sqlite::memory:").await.unwrap();
    let app = create_app(pool);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/tasks/00000000-0000-0000-0000-000000000000")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_task_invalid_uuid() {
    let pool = db::init("sqlite::memory:").await.unwrap();
    let app = create_app(pool);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/tasks/invalid-uuid")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_status_includes_task_counts() {
    let pool = db::init("sqlite::memory:").await.unwrap();

    // Create tasks in different states
    let running_task = create_task(&pool, "running").await.unwrap();
    mark_task_running(&pool, &running_task.id).await.unwrap();

    let _pending_task = create_task(&pool, "pending").await.unwrap();

    let failed_task = create_task(&pool, "failed").await.unwrap();
    fail_task(&pool, &failed_task.id, "error").await.unwrap();

    let app = create_app(pool);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["running_tasks"], 1);
    assert_eq!(json["pending_tasks"], 1);
    assert_eq!(json["failed_tasks"], 1);
}

#[tokio::test]
async fn test_full_task_lifecycle() {
    let pool = db::init("sqlite::memory:").await.unwrap();
    let app = create_app(pool.clone());

    // 1. Create a task via API
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/tasks")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"prompt": "test task"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let task_id = json["task_id"].as_str().unwrap();

    // 2. Verify task was created in pending state
    let task = get_task(&pool, &uuid::Uuid::parse_str(task_id).unwrap())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(task.status, TaskStatus::Pending);

    // 3. Verify status endpoint shows pending count
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["pending_tasks"], 1);
}

#[tokio::test]
async fn test_concurrent_task_creation() {
    let pool = db::init("sqlite::memory:").await.unwrap();
    let app = create_app(pool.clone());

    // Create multiple tasks concurrently
    let mut handles = vec![];
    for i in 0..10 {
        let app = app.clone();
        let handle = tokio::spawn(async move {
            app.oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/tasks")
                    .header("content-type", "application/json")
                    .body(Body::from(format!(r#"{{"prompt": "task {}"}}"#, i)))
                    .unwrap(),
            )
            .await
            .unwrap()
        });
        handles.push(handle);
    }

    // Wait for all creations
    let results = futures::future::join_all(handles).await;

    // All should succeed
    for result in results {
        assert_eq!(result.expect("task panicked").status(), StatusCode::CREATED);
    }

    // Verify all 10 tasks exist
    let tasks = get_all_tasks(&pool, 100).await.unwrap();
    assert_eq!(tasks.len(), 10);
}

#[tokio::test]
async fn test_worker_state_transitions() {
    let pool = db::init("sqlite::memory:").await.unwrap();

    // Create a task
    let task = create_task(&pool, "test task").await.unwrap();
    let task_id = task.id;

    // Verify initial state
    let task = get_task(&pool, &task_id).await.unwrap().unwrap();
    assert_eq!(task.status, TaskStatus::Pending);
    assert!(task.started_at.is_none());
    assert!(task.completed_at.is_none());

    // Mark as running
    mark_task_running(&pool, &task_id).await.unwrap();
    let task = get_task(&pool, &task_id).await.unwrap().unwrap();
    assert_eq!(task.status, TaskStatus::Running);
    assert!(task.started_at.is_some());
    assert!(task.completed_at.is_none());

    // Mark as completed
    complete_task(&pool, &task_id, "success output", 0)
        .await
        .unwrap();
    let task = get_task(&pool, &task_id).await.unwrap().unwrap();
    assert_eq!(task.status, TaskStatus::Completed);
    assert_eq!(task.response, Some("success output".to_string()));
    assert_eq!(task.exit_code, Some(0));
    assert!(task.completed_at.is_some());
}

#[tokio::test]
async fn test_task_response_storage() {
    let pool = db::init("sqlite::memory:").await.unwrap();
    let app = create_app(pool.clone());

    // Create a task
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/tasks")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"prompt": "generate code"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let task_id = json["task_id"].as_str().unwrap();

    // Simulate worker completing the task with response
    let uuid = uuid::Uuid::parse_str(task_id).unwrap();
    mark_task_running(&pool, &uuid).await.unwrap();
    complete_task(&pool, &uuid, "Generated Python code...", 0)
        .await
        .unwrap();

    // Fetch task and verify response
    let response = app
        .oneshot(
            Request::builder()
                .uri(&format!("/tasks/{}", task_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "completed");
    assert_eq!(json["response"], "Generated Python code...");
    assert_eq!(json["exit_code"], 0);
    assert!(json["started_at"].is_string());
    assert!(json["completed_at"].is_string());
}

#[tokio::test]
async fn test_empty_task_list() {
    let pool = db::init("sqlite::memory:").await.unwrap();
    let app = create_app(pool);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/tasks")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["tasks"].as_array().unwrap().is_empty());
    assert_eq!(json["total"], 0);
}

#[tokio::test]
async fn test_invalid_status_filter() {
    let pool = db::init("sqlite::memory:").await.unwrap();

    // Create some tasks
    create_task(&pool, "task 1").await.unwrap();
    create_task(&pool, "task 2").await.unwrap();

    let app = create_app(pool);

    // Request with invalid status filter should still return all tasks
    let response = app
        .oneshot(
            Request::builder()
                .uri("/tasks?status=invalid")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    // Should return all tasks since invalid status falls through to get_all_tasks
    assert_eq!(json["total"], 2);
}

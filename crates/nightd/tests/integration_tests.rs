use axum::body::Body;
use axum::http::{Request, StatusCode};
use nightd::api::create_app;
use nightd::db;
use nightd::db::create_test_pool;
use tower::util::ServiceExt;

#[tokio::test]
async fn test_status_endpoint() {
    let pool = create_test_pool().await;
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
    let pool = create_test_pool().await;
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
    let pool = create_test_pool().await;
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
    let pool = create_test_pool().await;
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
    let pool = create_test_pool().await;

    // Create some tasks
    db::create_task(&pool, "task 1").await.unwrap();
    db::create_task(&pool, "task 2").await.unwrap();

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
    let pool = create_test_pool().await;

    // Create tasks with different statuses
    let task = db::create_task(&pool, "pending task").await.unwrap();
    db::complete_task(&pool, &task.id, "done", 0).await.unwrap();
    db::create_task(&pool, "another pending").await.unwrap();

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
    let pool = create_test_pool().await;

    // Create many tasks
    for i in 0..10 {
        db::create_task(&pool, &format!("task {}", i))
            .await
            .unwrap();
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
    let pool = create_test_pool().await;

    let task = db::create_task(&pool, "test task").await.unwrap();
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
    let pool = create_test_pool().await;
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
    let pool = create_test_pool().await;
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
    let pool = create_test_pool().await;

    // Create tasks in different states
    let running_task = db::create_task(&pool, "running").await.unwrap();
    db::mark_task_running(&pool, &running_task.id)
        .await
        .unwrap();

    let _pending_task = db::create_task(&pool, "pending").await.unwrap();

    let failed_task = db::create_task(&pool, "failed").await.unwrap();
    db::fail_task(&pool, &failed_task.id, "error")
        .await
        .unwrap();

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
    let pool = create_test_pool().await;
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
    let task = nightd::db::get_task(&pool, &uuid::Uuid::parse_str(task_id).unwrap())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(task.status, nightd::models::TaskStatus::Pending);

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

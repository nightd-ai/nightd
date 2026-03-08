use axum::body::Body;
use axum::http::{Request, StatusCode};
use nightd::api::create_app;
use nightd::db;
use nightd::models;
use tower::util::ServiceExt;

#[tokio::test]
async fn test_status_endpoint() {
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

    assert!(json.get("status").is_some());
    assert!(json.get("running_tasks").is_some());
    assert!(json.get("pending_tasks").is_some());
    assert!(json.get("failed_tasks").is_some());
}

#[tokio::test]
async fn test_status_includes_task_counts() {
    let pool = db::init("sqlite::memory:").await.unwrap();

    let _pending_task1 = models::create_task(&pool, "test task 1").await.unwrap();
    let _pending_task2 = models::create_task(&pool, "test task 2").await.unwrap();
    let _pending_task3 = models::create_task(&pool, "test task 3").await.unwrap();
    let _pending_task4 = models::create_task(&pool, "test task 4").await.unwrap();

    let running_task = models::create_task(&pool, "running task").await.unwrap();
    models::mark_task_running(&pool, &running_task.id)
        .await
        .unwrap();

    let failed_task = models::create_task(&pool, "failed task").await.unwrap();
    models::fail_task(&pool, &failed_task.id, "test failure")
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
    assert_eq!(json["pending_tasks"], 4);
    assert_eq!(json["failed_tasks"], 1);
    assert_eq!(json["status"], "OK");
}

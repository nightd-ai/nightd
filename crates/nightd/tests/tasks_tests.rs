use axum::body::Body;
use axum::http::{Request, StatusCode};
use nightd::api;
use nightd::db;
use nightd::models::TaskStatus;
use std::path::PathBuf;
use tower::util::ServiceExt;

#[tokio::test]
async fn test_create_task_endpoint() {
    let pool = db::init(PathBuf::from(":memory:")).await.unwrap();
    let router = api::router(pool);

    let response = router
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
    let pool = db::init(PathBuf::from(":memory:")).await.unwrap();
    let router = api::router(pool);

    let response = router
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
    let pool = db::init(PathBuf::from(":memory:")).await.unwrap();

    nightd::models::create_task(&pool, "task 1").await.unwrap();
    nightd::models::create_task(&pool, "task 2").await.unwrap();

    let router = api::router(pool);

    let response = router
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
    let pool = db::init(PathBuf::from(":memory:")).await.unwrap();

    let task = nightd::models::create_task(&pool, "pending task")
        .await
        .unwrap();
    nightd::models::complete_task(&pool, &task.id, "done", 0)
        .await
        .unwrap();
    nightd::models::create_task(&pool, "another pending")
        .await
        .unwrap();

    let router = api::router(pool);

    let response = router
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
    let pool = db::init(PathBuf::from(":memory:")).await.unwrap();

    for i in 0..10 {
        nightd::models::create_task(&pool, &format!("task {}", i))
            .await
            .unwrap();
    }

    let router = api::router(pool);

    let response = router
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
    let pool = db::init(PathBuf::from(":memory:")).await.unwrap();

    let task = nightd::models::create_task(&pool, "test task")
        .await
        .unwrap();
    let task_id = task.id.to_string();

    let router = api::router(pool);

    let response = router
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
    let pool = db::init(PathBuf::from(":memory:")).await.unwrap();
    let router = api::router(pool);

    let response = router
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
    let pool = db::init(PathBuf::from(":memory:")).await.unwrap();
    let router = api::router(pool);

    let response = router
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
async fn test_full_task_lifecycle() {
    let pool = db::init(PathBuf::from(":memory:")).await.unwrap();
    let router = api::router(pool.clone());

    let response = router
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

    let task = nightd::models::get_task(&pool, &uuid::Uuid::parse_str(task_id).unwrap())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(task.status, TaskStatus::Pending);

    let response = router
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
    let pool = db::init(PathBuf::from(":memory:")).await.unwrap();
    let router = api::router(pool.clone());

    let mut handles = vec![];
    for i in 0..10 {
        let router = router.clone();
        let handle = tokio::spawn(async move {
            router
                .oneshot(
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

    let results = futures::future::join_all(handles).await;

    for result in results {
        assert_eq!(result.expect("task panicked").status(), StatusCode::CREATED);
    }

    let tasks = nightd::models::get_all_tasks(&pool, 100).await.unwrap();
    assert_eq!(tasks.len(), 10);
}

#[tokio::test]
async fn test_task_response_storage() {
    let pool = db::init(PathBuf::from(":memory:")).await.unwrap();
    let router = api::router(pool.clone());

    let response = router
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

    let uuid = uuid::Uuid::parse_str(task_id).unwrap();
    nightd::models::mark_task_running(&pool, &uuid)
        .await
        .unwrap();
    nightd::models::complete_task(&pool, &uuid, "Generated Python code...", 0)
        .await
        .unwrap();

    let response = router
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
    let pool = db::init(PathBuf::from(":memory:")).await.unwrap();
    let router = api::router(pool);

    let response = router
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
    let pool = db::init(PathBuf::from(":memory:")).await.unwrap();

    nightd::models::create_task(&pool, "task 1").await.unwrap();
    nightd::models::create_task(&pool, "task 2").await.unwrap();

    let router = api::router(pool);

    let response = router
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
    assert_eq!(json["total"], 2);
}

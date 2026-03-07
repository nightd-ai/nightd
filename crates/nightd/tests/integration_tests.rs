use axum::body::Body;
use axum::http::{Request, StatusCode};
use nightd::api::create_app;
use tower::util::ServiceExt;

#[tokio::test]
async fn test_status_endpoint() {
    let app = create_app();

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
    let app = create_app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    assert!(body_str.contains("OK"));
}

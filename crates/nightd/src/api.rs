use axum::{Json, Router, routing::get};
use serde::Serialize;

#[derive(Serialize)]
struct StatusResponse {
    status: String,
}

async fn status() -> Json<StatusResponse> {
    Json(StatusResponse {
        status: "OK".to_string(),
    })
}

pub fn create_app() -> Router {
    Router::new().route("/status", get(status))
}

use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct StatusResponse {
    status: String,
}

pub(crate) async fn get() -> Json<StatusResponse> {
    Json(StatusResponse {
        status: "OK".to_string(),
    })
}

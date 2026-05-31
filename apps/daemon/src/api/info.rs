use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct InfoResponse {
    pub(crate) status: String,
}

pub(crate) async fn info_handler() -> Json<InfoResponse> {
    Json(InfoResponse {
        status: "ok".to_string(),
    })
}

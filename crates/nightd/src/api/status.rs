use axum::{Json, extract::State};
use serde::Serialize;

use crate::api::{AppError, AppState};
use crate::models::{self, TaskStatus};

#[derive(Serialize)]
pub struct StatusResponse {
    pub status: String,
    pub running_tasks: i64,
    pub pending_tasks: i64,
    pub failed_tasks: i64,
}

pub(crate) async fn handler(
    State(state): State<AppState>,
) -> Result<Json<StatusResponse>, AppError> {
    let running_tasks = models::count_tasks_by_status(&state.db_pool, TaskStatus::Running).await?;
    let pending_tasks = models::count_tasks_by_status(&state.db_pool, TaskStatus::Pending).await?;
    let failed_tasks = models::count_tasks_by_status(&state.db_pool, TaskStatus::Failed).await?;

    Ok(Json(StatusResponse {
        status: "OK".to_string(),
        running_tasks,
        pending_tasks,
        failed_tasks,
    }))
}

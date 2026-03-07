use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use time::format_description::well_known::Rfc3339;
use uuid::Uuid;

use crate::models::{self, Task, TaskStatus};

// Custom error response type
struct AppError(StatusCode, String);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = (self.0, self.1);
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

// Custom JSON extractor that returns 422 for invalid JSON
async fn custom_json_extractor<T>(req: axum::extract::Request) -> Result<T, AppError>
where
    T: serde::de::DeserializeOwned,
{
    let (_parts, body) = req.into_parts();
    let bytes = match axum::body::to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(_) => {
            return Err(AppError(
                StatusCode::UNPROCESSABLE_ENTITY,
                "Invalid body".to_string(),
            ));
        }
    };

    match serde_json::from_slice::<T>(&bytes) {
        Ok(value) => Ok(value),
        Err(e) => Err(AppError(
            StatusCode::UNPROCESSABLE_ENTITY,
            format!("JSON error: {}", e),
        )),
    }
}

// State for the application
#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) db_pool: SqlitePool,
}

// Request/Response types
#[derive(Deserialize)]
pub(crate) struct CreateTaskRequest {
    pub(crate) prompt: String,
}

#[derive(Serialize)]
pub(crate) struct CreateTaskResponse {
    pub(crate) task_id: String,
    pub(crate) status: String,
}

#[derive(Serialize)]
pub(crate) struct TaskDto {
    pub(crate) id: String,
    pub(crate) prompt: String,
    pub(crate) status: String,
    pub(crate) response: Option<String>,
    pub(crate) exit_code: Option<i32>,
    pub(crate) created_at: String,
    pub(crate) started_at: Option<String>,
    pub(crate) completed_at: Option<String>,
}

impl From<Task> for TaskDto {
    fn from(task: Task) -> Self {
        Self {
            id: task.id.to_string(),
            prompt: task.prompt,
            status: task.status.to_string(),
            response: task.response,
            exit_code: task.exit_code,
            created_at: task.created_at.format(&Rfc3339).unwrap_or_default(),
            started_at: task
                .started_at
                .map(|t| t.format(&Rfc3339).unwrap_or_default()),
            completed_at: task
                .completed_at
                .map(|t| t.format(&Rfc3339).unwrap_or_default()),
        }
    }
}

#[derive(Serialize)]
pub(crate) struct TaskListResponse {
    pub(crate) tasks: Vec<TaskDto>,
    pub(crate) total: usize,
}

#[derive(Serialize)]
pub(crate) struct StatusResponse {
    pub(crate) status: String,
    pub(crate) running_tasks: i64,
    pub(crate) pending_tasks: i64,
    pub(crate) failed_tasks: i64,
}

#[derive(Deserialize)]
pub(crate) struct ListTasksQuery {
    pub(crate) status: Option<String>,
    pub(crate) limit: Option<i64>,
}

// Handler functions
async fn status(State(state): State<AppState>) -> Result<Json<StatusResponse>, StatusCode> {
    let running_count = models::count_tasks_by_status(&state.db_pool, TaskStatus::Running)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let pending_count = models::count_tasks_by_status(&state.db_pool, TaskStatus::Pending)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let failed_count = models::count_tasks_by_status(&state.db_pool, TaskStatus::Failed)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(StatusResponse {
        status: "OK".to_string(),
        running_tasks: running_count,
        pending_tasks: pending_count,
        failed_tasks: failed_count,
    }))
}

async fn create_task(
    State(state): State<AppState>,
    req: axum::extract::Request,
) -> Result<(StatusCode, Json<CreateTaskResponse>), AppError> {
    let request: CreateTaskRequest = custom_json_extractor(req).await?;

    let task = models::create_task(&state.db_pool, &request.prompt)
        .await
        .map_err(|_| {
            AppError(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            )
        })?;

    Ok((
        StatusCode::CREATED,
        Json(CreateTaskResponse {
            task_id: task.id.to_string(),
            status: task.status.to_string(),
        }),
    ))
}

async fn list_tasks(
    State(state): State<AppState>,
    Query(query): Query<ListTasksQuery>,
) -> Result<Json<TaskListResponse>, StatusCode> {
    let limit = query.limit.unwrap_or(20);

    let tasks = match query.status.as_deref() {
        Some("pending") => {
            models::get_tasks_by_status(&state.db_pool, TaskStatus::Pending, limit).await
        }
        Some("running") => {
            models::get_tasks_by_status(&state.db_pool, TaskStatus::Running, limit).await
        }
        Some("completed") => {
            models::get_tasks_by_status(&state.db_pool, TaskStatus::Completed, limit).await
        }
        Some("failed") => {
            models::get_tasks_by_status(&state.db_pool, TaskStatus::Failed, limit).await
        }
        _ => models::get_all_tasks(&state.db_pool, limit).await,
    }
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total = tasks.len();
    let task_dtos: Vec<TaskDto> = tasks.into_iter().map(TaskDto::from).collect();

    Ok(Json(TaskListResponse {
        tasks: task_dtos,
        total,
    }))
}

async fn get_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskDto>, StatusCode> {
    let uuid = Uuid::parse_str(&task_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    let task = models::get_task(&state.db_pool, &uuid)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(TaskDto::from(task)))
}

// Create the router with state
pub fn create_app(db_pool: SqlitePool) -> Router {
    let state = AppState { db_pool };

    Router::new()
        .route("/status", get(status))
        .route("/tasks", post(create_task).get(list_tasks))
        .route("/tasks/{task_id}", get(get_task))
        .with_state(state)
}

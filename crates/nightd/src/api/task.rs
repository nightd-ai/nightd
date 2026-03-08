use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use time::format_description::well_known::Rfc3339;
use uuid::Uuid;

use crate::api::{AppError, AppState};
use crate::models::{self, Task, TaskStatus};

#[derive(Deserialize)]
pub(crate) struct CreateTaskRequest {
    pub prompt: String,
}

#[derive(Serialize)]
pub(crate) struct CreateTaskResponse {
    pub task_id: String,
    pub status: String,
}

#[derive(Serialize)]
pub(crate) struct TaskDto {
    pub id: String,
    pub prompt: String,
    pub status: String,
    pub response: Option<String>,
    pub exit_code: Option<i32>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct TaskListResponse {
    pub tasks: Vec<TaskDto>,
    pub total: usize,
}

#[derive(Deserialize)]
pub(crate) struct ListTasksQuery {
    pub status: Option<String>,
    pub limit: Option<i64>,
}

impl From<Task> for TaskDto {
    fn from(task: Task) -> Self {
        TaskDto {
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

fn extract_json<T>(
    result: Result<Json<T>, axum::extract::rejection::JsonRejection>,
) -> Result<T, AppError> {
    result.map(|Json(payload)| payload).map_err(|e| {
        AppError::new(
            "Invalid JSON",
            StatusCode::UNPROCESSABLE_ENTITY,
            Some(format!("Failed to parse request body: {}", e)),
        )
    })
}

pub(crate) async fn create_handler(
    State(state): State<AppState>,
    result: Result<Json<CreateTaskRequest>, axum::extract::rejection::JsonRejection>,
) -> Result<(StatusCode, Json<CreateTaskResponse>), AppError> {
    let request = extract_json(result)?;
    let task = models::create_task(&state.db_pool, &request.prompt).await?;

    Ok((
        StatusCode::CREATED,
        Json(CreateTaskResponse {
            task_id: task.id.to_string(),
            status: task.status.to_string(),
        }),
    ))
}

pub(crate) async fn list_handler(
    State(state): State<AppState>,
    Query(query): Query<ListTasksQuery>,
) -> Result<Json<TaskListResponse>, AppError> {
    let status = query
        .status
        .and_then(|s| TaskStatus::try_from(s.as_str()).ok());
    let limit = query.limit.unwrap_or(20);
    let tasks = if let Some(status) = status {
        models::get_tasks_by_status(&state.db_pool, status, limit).await?
    } else {
        models::get_all_tasks(&state.db_pool, limit).await?
    };
    let total = tasks.len();
    let task_dtos: Vec<TaskDto> = tasks.into_iter().map(TaskDto::from).collect();

    Ok(Json(TaskListResponse {
        tasks: task_dtos,
        total,
    }))
}

pub(crate) async fn get_handler(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<TaskDto>, AppError> {
    let task = models::get_task(&state.db_pool, &task_id).await?;

    match task {
        Some(task) => Ok(Json(TaskDto::from(task))),
        None => Err(AppError::new(
            "Task not found",
            StatusCode::NOT_FOUND,
            Some(format!("Task with ID {} does not exist", task_id)),
        )),
    }
}

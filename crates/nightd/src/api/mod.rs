use axum::{
    Json, Router,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::Serialize;
use sqlx::SqlitePool;
use std::net::SocketAddr;

mod status;
mod task;

#[derive(Serialize)]
pub struct AppError {
    r#type: String,
    title: String,
    status: u16,
    detail: String,
}

impl AppError {
    pub fn new(title: &str, status: StatusCode, detail: Option<String>) -> Self {
        Self {
            r#type: "about:blank".to_string(),
            title: title.to_string(),
            status: status.as_u16(),
            detail: detail.unwrap_or_default(),
        }
    }

    pub fn with_status(status: StatusCode, detail: String) -> Self {
        Self {
            r#type: "about:blank".to_string(),
            title: "Error".to_string(),
            status: status.as_u16(),
            detail,
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::new(
            "Database error",
            StatusCode::INTERNAL_SERVER_ERROR,
            Some(err.to_string()),
        )
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, Json(self)).into_response()
    }
}

#[derive(Clone)]
pub struct App {
    db_pool: SqlitePool,
}

pub fn router(db_pool: SqlitePool) -> Router {
    let state = App { db_pool };

    Router::new()
        .route("/status", get(status::handler))
        .route("/tasks", post(task::create_handler))
        .route("/tasks", get(task::list_handler))
        .route("/tasks/{task_id}", get(task::get_handler))
        .with_state(state)
}

pub(crate) async fn run(app: Router, host: &str, port: u16) -> std::io::Result<()> {
    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .expect("Failed to parse address");

    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let mut sigterm =
            signal(SignalKind::terminate()).expect("Failed to create SIGTERM handler");
        let mut sigint = signal(SignalKind::interrupt()).expect("Failed to create SIGINT handler");

        tokio::select! {
            _ = sigterm.recv() => {},
            _ = sigint.recv() => {},
        }
    }

    #[cfg(windows)]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}

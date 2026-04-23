use axum::{Router, routing::get};
use sqlx::SqlitePool;
use tokio::sync::mpsc;

pub mod status;

#[derive(Clone)]
#[allow(dead_code)]
pub(crate) struct AppContext {
    pub(crate) db: SqlitePool,
    pub(crate) session_tx: mpsc::Sender<agent::Session>,
}

pub fn router(db: SqlitePool, session_tx: mpsc::Sender<agent::Session>) -> Router {
    Router::new()
        .route("/status", get(status::get))
        .with_state(AppContext { db, session_tx })
}

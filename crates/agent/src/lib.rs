use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::{Semaphore, mpsc};
use tokio_util::sync::CancellationToken;

#[derive(Clone, Debug)]
pub struct Session {
    #[allow(dead_code)]
    pub(crate) id: uuid::Uuid,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::now_v7(),
        }
    }
}

pub async fn run_executor(db: &SqlitePool) -> (mpsc::Sender<Session>, CancellationToken) {
    let shutdown_token = CancellationToken::new();
    let semaphore = Arc::new(Semaphore::new(5));
    let (session_tx, mut session_rx) = mpsc::channel(100);
    let db = db.clone();
    let shutdown = shutdown_token.clone();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                biased;
                _ = shutdown.cancelled() => {
                    break;
                }
                Some(session) = session_rx.recv() => {
                    let semaphore = semaphore.clone();
                    let db = db.clone();
                    tokio::spawn(async move {
                        let _ = semaphore
                            .acquire_owned()
                            .await
                            .expect("semaphore closed");
                        execute_session(db, session).await;
                    });
                }
            }
        }
    });
    (session_tx, shutdown_token)
}

async fn execute_session(_db: SqlitePool, _session: Session) {
    // TODO: implement session execution
}

pub mod error;

pub use error::{Error, Result};

use sqlx::SqlitePool;
use std::path::PathBuf;

pub struct Executor {
    _db: SqlitePool,
    _app_data_dir: PathBuf,
}

impl Executor {
    pub fn new(db: SqlitePool, app_data_dir: PathBuf) -> Self {
        Self {
            _db: db,
            _app_data_dir: app_data_dir,
        }
    }

    pub async fn run_loop(&self) -> Result<()> {
        todo!()
    }

    pub async fn execute_one(&self, session_id: uuid::Uuid) -> Result<()> {
        let _ = session_id;
        todo!()
    }
}

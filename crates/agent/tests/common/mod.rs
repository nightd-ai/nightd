//! Common test utilities for agent crate tests.

use std::path::PathBuf;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use tempfile::TempDir;

use agent::Scheduler;

/// Test context providing shared resources for tests.
///
/// This struct manages the lifecycle of test infrastructure including
/// a SQLite database and temporary directories.
pub struct TestContext {
    /// The scheduler instance for managing sessions.
    pub scheduler: Scheduler,
    /// Temporary directory for test workspaces and database.
    pub temp_dir: TempDir,
    /// Database pool for direct database access.
    pub db_pool: SqlitePool,
}

impl TestContext {
    /// Create a new test context with a SQLite database.
    ///
    /// This creates:
    /// - A temporary directory for workspace storage and database
    /// - A file-based SQLite database (not in-memory, to support migrations)
    /// - A scheduler instance with the database
    pub async fn new() -> anyhow::Result<Self> {
        // Create a temporary directory for test workspaces and database
        let temp_dir = TempDir::new()?;

        // Create a file-based SQLite database in the temp directory
        // Using file-based instead of in-memory to support sqlx migrations
        let db_path = temp_dir.path().join("test.db");

        let options = SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true);

        let db_pool = SqlitePool::connect_with(options).await?;

        // Create scheduler - this runs migrations
        // Note: migrations use a relative path, so tests must be run from the workspace root
        let scheduler = Scheduler::new(db_pool.clone()).await?;

        Ok(Self {
            scheduler,
            temp_dir,
            db_pool,
        })
    }

    /// Create a test workspace directory.
    ///
    /// Returns the path to a newly created subdirectory within the temp directory.
    pub fn create_workspace(&self, name: &str) -> PathBuf {
        let path = self.temp_dir.path().join(name);
        std::fs::create_dir_all(&path).expect("Failed to create workspace directory");
        path
    }

    /// Create a test workspace with a file.
    ///
    /// Creates a workspace directory with an optional initial file.
    #[allow(dead_code)]
    pub fn create_workspace_with_file(&self, name: &str, filename: &str, content: &str) -> PathBuf {
        let path = self.create_workspace(name);
        let file_path = path.join(filename);
        std::fs::write(&file_path, content).expect("Failed to write test file");
        path
    }

    /// Get a reference to the database pool.
    pub fn db(&self) -> &SqlitePool {
        &self.db_pool
    }

    /// Get the path to the temp directory.
    #[allow(dead_code)]
    pub fn temp_path(&self) -> &std::path::Path {
        self.temp_dir.path()
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        // TempDir is automatically cleaned up when dropped
        // Database file is in temp dir, so it's also cleaned up
    }
}

/// Setup function to initialize tracing for tests.
#[allow(dead_code)]
pub fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();
}

/// Fake agent module for testing.
pub mod fake_agent;

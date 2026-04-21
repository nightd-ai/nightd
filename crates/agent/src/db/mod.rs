//! Database module for agent persistence.
//!
//! This module handles database connections, migrations, and queries.

use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use std::str::FromStr;

pub mod migrations;

/// Database connection pool.
#[derive(Debug, Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Create a new database connection pool.
    pub async fn new(database_url: &str) -> crate::Result<Self> {
        let options = SqliteConnectOptions::from_str(database_url)?.create_if_missing(true);

        let pool = SqlitePool::connect_with(options).await?;

        // Run migrations
        sqlx::migrate!("./src/db/migrations").run(&pool).await?;

        Ok(Self { pool })
    }

    /// Get a reference to the connection pool.
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

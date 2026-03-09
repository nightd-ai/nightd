use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use std::path::PathBuf;

pub async fn init(database: PathBuf) -> Result<SqlitePool, sqlx::Error> {
    // Handle in-memory database specially
    let database_url = if database.as_os_str() == ":memory:" {
        "sqlite::memory:".to_string()
    } else {
        // Resolve relative paths to data directory
        let database_path = if database.is_relative() {
            dirs::data_dir()
                .map(|d| d.join("nightd").join(&database))
                .unwrap_or_else(|| database)
        } else {
            database
        };

        // Create parent directories if they don't exist
        if let Some(parent) = database_path.parent()
            && !parent.exists()
        {
            std::fs::create_dir_all(parent).map_err(|e| {
                sqlx::Error::Protocol(format!("Failed to create database directory: {}", e))
            })?;
        }

        // Build database URL
        format!("sqlite://{}", database_path.display())
    };

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .map_err(|e| sqlx::Error::Migrate(Box::new(e)))?;

    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_initialization() {
        let pool = init(PathBuf::from(":memory:")).await.unwrap();

        // Verify tasks table exists
        let result: Result<i64, _> = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='tasks'",
        )
        .fetch_one(&pool)
        .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);

        // Verify index exists
        let result: Result<i64, _> = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name='idx_status_created'",
        )
        .fetch_one(&pool)
        .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);

        // Verify we can query the tasks table
        let result: Result<i64, _> = sqlx::query_scalar("SELECT COUNT(*) FROM tasks")
            .fetch_one(&pool)
            .await;
        assert!(result.is_ok());
    }
}

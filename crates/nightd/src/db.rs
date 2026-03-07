use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

pub async fn init(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
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
        let pool = init("sqlite::memory:").await.unwrap();

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

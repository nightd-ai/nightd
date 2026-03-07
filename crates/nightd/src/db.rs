use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use std::path::Path;

#[allow(dead_code)]
pub(crate) async fn init_pool(database_path: &Path) -> Result<SqlitePool, sqlx::Error> {
    let database_url = format!("sqlite://{}", database_path.display());

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    Ok(pool)
}

#[allow(dead_code)]
pub(crate) async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::migrate!("../../migrations")
        .run(pool)
        .await
        .map_err(|e| sqlx::Error::Migrate(Box::new(e)))
}

#[cfg(test)]
pub(crate) async fn create_test_pool() -> SqlitePool {
    let pool = sqlx::sqlite::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to create test database pool");

    sqlx::migrate!("../../migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations on test database");

    pool
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_database_initialization() {
        let pool = create_test_pool().await;
        let result: Result<i64, _> = sqlx::query_scalar("SELECT COUNT(*) FROM tasks")
            .fetch_one(&pool)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_uuid_format() {
        let uuid = Uuid::new_v4();
        let uuid_str = uuid.to_string();

        // Verify UUID v4 format (should have dashes and proper version)
        assert_eq!(uuid_str.len(), 36);
        assert!(uuid_str.contains('-'));

        // Verify it's a valid UUID v4
        let parsed = Uuid::parse_str(&uuid_str).unwrap();
        assert_eq!(parsed.get_version_num(), 4);
    }

    #[tokio::test]
    async fn test_migrations_run_successfully() {
        let pool = create_test_pool().await;

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
    }
}

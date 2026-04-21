use sacp::ContentBlock;
use sqlx::SqlitePool;
use uuid::Uuid;

pub async fn store_updates(
    pool: &SqlitePool,
    session_id: Uuid,
    updates: &[ContentBlock],
) -> crate::Result<()> {
    let _ = pool;
    let _ = session_id;
    let _ = updates;
    todo!()
}

pub async fn get_result(pool: &SqlitePool, session_id: Uuid) -> crate::Result<Option<String>> {
    let _ = pool;
    let _ = session_id;
    todo!()
}

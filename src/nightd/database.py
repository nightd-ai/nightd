import aiosqlite
from typing import Optional

from nightd.config import settings


def get_db_path() -> str:
    """Return the database path from config."""
    return settings.database_path


async def init_db() -> None:
    """Create the tasks table if it doesn't exist."""
    async with aiosqlite.connect(get_db_path()) as db:
        await db.execute("""
            CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                description TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                result TEXT,
                error_message TEXT
            )
        """)
        await db.commit()


async def create_task(task_id: str, description: str) -> None:
    """Insert a new task with status 'pending'."""
    async with aiosqlite.connect(get_db_path()) as db:
        await db.execute(
            """
            INSERT INTO tasks (id, description, status)
            VALUES (?, ?, ?)
            """,
            (task_id, description, "pending"),
        )
        await db.commit()


async def update_task_status(
    task_id: str,
    status: str,
    result: Optional[str] = None,
    error_message: Optional[str] = None,
) -> None:
    """Update task status."""
    async with aiosqlite.connect(get_db_path()) as db:
        await db.execute(
            """
            UPDATE tasks
            SET status = ?,
                result = ?,
                error_message = ?,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            """,
            (status, result, error_message, task_id),
        )
        await db.commit()


async def get_task(task_id: str) -> Optional[dict]:
    """Get task by ID."""
    async with aiosqlite.connect(get_db_path()) as db:
        db.row_factory = aiosqlite.Row
        async with db.execute("SELECT * FROM tasks WHERE id = ?", (task_id,)) as cursor:
            row = await cursor.fetchone()
            if row is None:
                return None
            return dict(row)

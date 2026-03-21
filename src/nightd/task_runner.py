import asyncio
import uuid
import logging

from .database import init_db, create_task, update_task_status
from .agents.orchestrator import run_workflow


logger = logging.getLogger(__name__)


async def execute_task(task_id: str, description: str):
    logger.info(f"Starting task execution: {task_id}")

    await update_task_status(task_id, "running")

    try:
        result = await run_workflow(description)
        result_json = str(result) if not isinstance(result, str) else result
        await update_task_status(task_id, "completed", result=result_json)
        logger.info(f"Task completed successfully: {task_id}")
    except Exception as e:
        error_message = str(e)
        await update_task_status(task_id, "failed", error_message=error_message)
        logger.error(f"Task failed: {task_id} - {error_message}")


def run_task(task_id: str, description: str):
    """Synchronous wrapper for running async execute_task in background."""
    asyncio.run(execute_task(task_id, description))


async def start_task(description: str) -> str:
    task_id = str(uuid.uuid4())

    await init_db()
    await create_task(task_id, description)

    return task_id

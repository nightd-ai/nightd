from fastapi import FastAPI, BackgroundTasks, HTTPException

from .models.task import TaskCreate, TaskCreateResponse, TaskResponse
from .task_runner import start_task, run_task
from .database import get_task as db_get_task

app = FastAPI()


@app.get("/status")
async def status():
    return {"status": "OK"}


@app.post("/tasks")
async def create_task(task: TaskCreate, background_tasks: BackgroundTasks):
    task_id = await start_task(task.description)
    background_tasks.add_task(run_task, task_id, task.description)
    return TaskCreateResponse(task_id=task_id, status="pending")


@app.get("/tasks/{task_id}")
async def get_task_by_id(task_id: str):
    task = await db_get_task(task_id)
    if task is None:
        raise HTTPException(status_code=404, detail="Task not found")
    task["task_id"] = task.pop("id")
    return TaskResponse(**task)

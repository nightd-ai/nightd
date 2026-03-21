from typing import Literal, Optional

from pydantic import BaseModel, Field


class TaskCreate(BaseModel):
    description: str = Field(..., min_length=1, description="Description of the task")


class TaskResponse(BaseModel):
    task_id: str = Field(..., description="Unique identifier for the task")
    description: str = Field(..., description="Description of the task")
    status: Literal["pending", "running", "completed", "failed"] = Field(
        ..., description="Current status of the task"
    )
    created_at: str = Field(
        ..., description="ISO format datetime when task was created"
    )
    updated_at: str = Field(
        ..., description="ISO format datetime when task was last updated"
    )
    result: Optional[str] = Field(None, description="Result of the task if completed")
    error_message: Optional[str] = Field(
        None, description="Error message if task failed"
    )


class TaskCreateResponse(BaseModel):
    task_id: str = Field(..., description="Unique identifier for the created task")
    status: str = Field(..., description="Initial status of the task")

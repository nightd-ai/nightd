"""Orchestrator agent that coordinates the workflow between agents using Pydantic AI."""

import logging

import mlflow
import mlflow.pydantic_ai
from pydantic_ai import Agent, RunContext

from nightd.agents.spec_writer import write_spec
from nightd.agents.coder import implement_spec
from nightd.agents.reviewer import review_changes
from nightd.config import settings

logger = logging.getLogger(__name__)

# Enable MLflow tracing for Pydantic AI
mlflow.set_tracking_uri(settings.mlflow_tracking_uri)
mlflow.set_experiment(settings.mlflow_experiment_name)
mlflow.pydantic_ai.autolog()

orchestrator_agent = Agent(
    model="claude-sonnet-4-6",
    system_prompt=(
        "You are a workflow orchestrator agent. Your role is to coordinate the software "
        "development workflow by intelligently deciding when to call each specialized agent. "
        "You have access to three tools:\n\n"
        "1. write_spec: Creates an implementation plan/specification for a given task\n"
        "2. implement_spec: Implements code changes based on a specification\n"
        "3. review_changes: Reviews implemented code against the original specification\n\n"
        "Analyze the task and use the appropriate tools to complete the workflow. "
        "You can call tools multiple times if needed, iterate on work, or skip steps "
        "that don't make sense for the task. Be flexible and intelligent in your approach."
    ),
    instrument=True,
)


@orchestrator_agent.tool
async def write_spec_tool(ctx: RunContext, task_description: str) -> str:
    """Create an implementation plan/specification for a coding task.

    Args:
        task_description: Description of the coding task to analyze.

    Returns:
        The generated implementation plan/spec text.
    """
    logger.info("Orchestrator calling write_spec tool")
    return await write_spec(task_description)


@orchestrator_agent.tool
async def implement_spec_tool(ctx: RunContext, spec: str) -> str:
    """Implement code changes based on the provided specification.

    Args:
        spec: The specification describing the changes to implement.

    Returns:
        Implementation summary of what was done.
    """
    logger.info("Orchestrator calling implement_spec tool")
    return await implement_spec(spec)


@orchestrator_agent.tool
async def review_changes_tool(
    ctx: RunContext, spec: str, implementation_summary: str
) -> str:
    """Review implemented code changes against the original specification.

    Args:
        spec: The original specification describing what was to be implemented.
        implementation_summary: Summary of the implementation that was completed.

    Returns:
        Review text with feedback on the implementation.
    """
    logger.info("Orchestrator calling review_changes tool")
    return await review_changes(spec, implementation_summary)


async def run_workflow(task_description: str) -> dict:
    """Run the workflow orchestrator to complete a task.

    Args:
        task_description: Description of the task to be completed.

    Returns:
        Dict containing the orchestrator's response and any tool results.
    """
    logger.info(
        "Starting orchestrated workflow for task: %s", task_description[:50] + "..."
    )

    result = await orchestrator_agent.run(task_description)

    logger.info("Workflow completed")

    return {
        "response": result.output,
        "messages": result.messages if hasattr(result, "messages") else None,
    }

"""Spec writer agent for creating implementation plans."""

import logging

from nightd.agents.base import create_agent_client

SYSTEM_PROMPT = (
    "You are a spec writer agent. Analyze the coding task and create a detailed "
    "implementation plan. The plan should include: 1) What files need to be changed, "
    "2) What the changes should accomplish, 3) Any considerations or edge cases. "
    "Use the available tools to explore the codebase if needed."
)

logger = logging.getLogger(__name__)


async def write_spec(task_description: str) -> str:
    """Analyze a coding task and create an implementation plan.

    Args:
        task_description: Description of the coding task to analyze.

    Returns:
        The generated implementation plan/spec text, or an error message if the agent fails.
    """
    try:
        logger.info(f"Starting spec writing for task: {task_description[:100]}...")

        async with create_agent_client(system_prompt=SYSTEM_PROMPT) as client:
            response = await client.run(task_description)
            logger.info("Spec writing completed successfully")
            return response

    except Exception as e:
        error_msg = f"Error generating spec: {str(e)}"
        logger.error(error_msg)
        return error_msg

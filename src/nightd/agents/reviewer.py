"""Reviewer agent for reviewing code changes against specifications."""

import logging

from nightd.agents.base import create_agent_client

logger = logging.getLogger(__name__)

SYSTEM_PROMPT = (
    "You are a code reviewer agent. Review the implementation against the original spec. "
    "Check for: 1) Correctness - does it do what was asked, 2) Code quality - best practices, "
    "readability, 3) Security - any potential issues, 4) Edge cases - proper error handling. "
    "Provide specific feedback on what was done well and what could be improved."
)


async def review_changes(spec: str, implementation_summary: str) -> str:
    """Review implemented code changes against the original specification.

    Args:
        spec: The original specification describing what was to be implemented.
        implementation_summary: Summary of the implementation that was completed.

    Returns:
        Review text with feedback on the implementation, or an error message if the agent fails.
    """
    try:
        prompt = (
            f"Original Specification:\n{spec}\n\n"
            f"Implementation Summary:\n{implementation_summary}\n\n"
            "Please review the implementation against the original specification."
        )

        logger.info(f"Sending review request for spec: {spec[:100]}...")

        async with create_agent_client(system_prompt=SYSTEM_PROMPT) as client:
            response = await client.run(prompt)
            logger.info("Code review completed successfully")
            return response

    except Exception as e:
        error_msg = f"Reviewer agent failed: {str(e)}"
        logger.error(error_msg)
        return error_msg

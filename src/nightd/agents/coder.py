"""Coder agent for implementing code changes based on a spec."""

import logging

from nightd.agents.base import create_agent_client

logger = logging.getLogger(__name__)


async def implement_spec(spec: str) -> str:
    """Implement code changes based on the provided spec.

    Args:
        spec: The specification describing the changes to implement.

    Returns:
        Implementation summary or error message if the agent fails.
    """
    system_prompt = (
        "You are a coder agent. Implement the changes described in the spec. "
        "Use the available tools (Read, Write, Edit, Bash) to make the necessary "
        "code changes. Follow best practices and ensure the code is correct. "
        "After making changes, verify they work as expected."
    )

    try:
        logger.info(f"Sending spec to coder agent: {spec[:100]}...")

        async with create_agent_client(system_prompt=system_prompt) as client:
            response = await client.run(spec)

        logger.info(f"Coder agent response: {response[:100]}...")

        return response
    except Exception as e:
        error_msg = f"Coder agent failed to implement spec: {str(e)}"
        logger.error(error_msg)
        return error_msg

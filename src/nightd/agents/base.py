"""Base agent client setup using Claude Agent SDK."""

import os
from collections.abc import AsyncIterator
from contextlib import asynccontextmanager
from typing import AsyncGenerator

from claude_agent_sdk import ClaudeSDKClient
from claude_agent_sdk.types import ClaudeAgentOptions

from nightd.config import settings

# Ensure the API key is available in the environment for the SDK
os.environ.setdefault("ANTHROPIC_API_KEY", settings.anthropic_api_key)

import mlflow
import mlflow.anthropic

# Configure MLflow with settings
mlflow.set_tracking_uri(settings.mlflow_tracking_uri)
mlflow.set_experiment(settings.mlflow_experiment_name)
mlflow.anthropic.autolog()

STANDARD_TOOLS = ["Read", "Write", "Edit", "Bash", "WebSearch"]


class AgentClient:
    """Async context manager for agent interactions."""

    def __init__(self, system_prompt: str | None = None):
        self.system_prompt = system_prompt
        self._response_parts: list[str] = []
        self._sdk_client: ClaudeSDKClient | None = None

    async def run(self, prompt: str) -> str:
        """Run the agent with the given prompt.

        Args:
            prompt: The user prompt to send to the agent.

        Returns:
            The agent's response as a string.
        """
        options = ClaudeAgentOptions(
            model="claude-sonnet-4-6",
            system_prompt=self.system_prompt,
            tools=STANDARD_TOOLS,
            permission_mode="acceptEdits",
        )

        self._response_parts = []

        # Use ClaudeSDKClient which supports MLflow tracing
        async with ClaudeSDKClient(options=options) as client:
            await client.query(prompt)
            async for message in client.receive_response():
                # Accumulate response parts
                if hasattr(message, "content") and message.content:
                    self._response_parts.append(str(message.content))
                elif isinstance(message, str):
                    self._response_parts.append(message)

        return "".join(self._response_parts)


@asynccontextmanager
async def create_agent_client(
    system_prompt: str | None = None,
) -> AsyncGenerator[AgentClient, None]:
    """Create an agent client as an async context manager.

    Args:
        system_prompt: Optional system prompt to configure the agent's behavior.

    Yields:
        An AgentClient instance for running agent interactions.

    Example:
        async with create_agent_client(system_prompt="You are a helpful assistant") as client:
            response = await client.run("What files are in the current directory?")
    """
    client = AgentClient(system_prompt=system_prompt)
    try:
        yield client
    finally:
        pass


async def run_agent(
    prompt: str,
    system_prompt: str | None = None,
) -> AsyncIterator:
    """Run an agent with the given prompt and configuration.

    Args:
        prompt: The user prompt to send to the agent.
        system_prompt: Optional system prompt to configure the agent's behavior.

    Yields:
        Messages and events from the agent interaction.

    Example:
        async for message in run_agent("What files are in the current directory?"):
            print(message)
    """
    options = ClaudeAgentOptions(
        model="claude-sonnet-4-6",
        system_prompt=system_prompt,
        tools=STANDARD_TOOLS,
        permission_mode="acceptEdits",
    )

    # Use ClaudeSDKClient which supports MLflow tracing
    async with ClaudeSDKClient(options=options) as client:
        await client.query(prompt)
        async for message in client.receive_response():
            yield message

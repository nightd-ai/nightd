import os
from typing import Any

import mlflow

mlflow.anthropic.autolog()


def setup_mlflow() -> None:
    """Configure MLflow tracking URI from environment variable."""
    tracking_uri = os.getenv("MLFLOW_TRACKING_URI", "http://localhost:5000")
    mlflow.set_tracking_uri(tracking_uri)
    mlflow.set_experiment("agent-evaluation")


def create_agent_options(
    tools: list[Any],
    system_prompt: str,
    max_turns: int = 10,
    cwd: str | None = None,
) -> Any:
    """Create ClaudeAgentOptions configuration.

    Args:
        tools: List of tools available to the agent.
        system_prompt: System prompt defining agent behavior.
        max_turns: Maximum number of turns.
        cwd: Working directory for the agent.

    Returns:
        ClaudeAgentOptions configuration object.
    """
    from claude_agent_sdk import ClaudeAgentOptions

    options = ClaudeAgentOptions(
        tools=tools,
        system_prompt=system_prompt,
        cwd=cwd or str(os.getcwd()),
        max_turns=max_turns,
    )
    return options


async def run_agent(
    options: Any,
    query: str,
) -> str:
    """Run the agent with a query and return the result.

    Args:
        options: ClaudeAgentOptions configuration.
        query: User query to run against the agent.

    Returns:
        The agent's final response.
    """
    from claude_agent_sdk import ClaudeSDKClient

    messages = []
    async with ClaudeSDKClient(options=options) as client:
        await client.query(query)
        async for message in client.receive_response():
            messages.append(message)

    return messages[-1].result if messages else ""


def run_agent_with_timeout(
    query: str,
    options: Any,
    timeout: int = 300,
) -> str:
    """Run agent with a timeout.

    Args:
        query: User query.
        options: ClaudeAgentOptions configuration.
        timeout: Timeout in seconds.

    Returns:
        Agent response.
    """
    import asyncio

    async def run_with_timeout():
        return await asyncio.wait_for(run_agent(options, query), timeout=timeout)

    return asyncio.run(run_with_timeout())


def create_dataset(
    name: str,
    experiment_id: str,
    data: list[dict[str, Any]],
    tags: dict[str, str] | None = None,
) -> Any:
    """Create an MLflow evaluation dataset.

    Args:
        name: Dataset name.
        experiment_id: Experiment ID to associate with dataset.
        data: List of evaluation cases with inputs and expectations.
        tags: Optional tags for the dataset.

    Returns:
        MLflow dataset object.
    """
    from mlflow.genai.datasets import create_dataset

    dataset = create_dataset(
        name=name,
        experiment_id=[experiment_id],
        tags=tags or {},
    )
    dataset.merge_records(data)
    return dataset


def make_retrieval_judge(model: str = "openai:/gpt-4o") -> Any:
    """Create a judge to evaluate retrieval redundancy.

    Args:
        model: Model to use for the judge.

    Returns:
        Redundancy judge.
    """
    from mlflow.genai.judges import make_judge

    return make_judge(
        name="retrieval_redundancy",
        model=model,
        instructions=(
            "Analyze {{ trace }} to check if there are redundant retrieval calls. "
            "Look at the source IDs returned from the retrieval tool. "
            "If multiple retrieval calls have the same source IDs, there is likely a redundancy. "
            "Return 'pass' if there are no redundant calls, 'fail' if there are redundant calls."
        ),
    )


def make_comprehensiveness_judge(model: str = "openai:/gpt-4o-mini") -> Any:
    """Create a judge to evaluate response comprehensiveness.

    Args:
        model: Model to use for the judge.

    Returns:
        Comprehensiveness judge.
    """
    from mlflow.genai.judges import make_judge

    return make_judge(
        name="comprehensive",
        model=model,
        instructions=(
            "Evaluate if the outputs comprehensively covers all relevant aspects for the query in the inputs. "
            "Return 'pass' if the output is comprehensive or 'fail' if not."
            "{{ outputs }}"
            "{{ expectations }}"
        ),
    )


def create_num_retrieval_calls_scorer() -> Any:
    """Create a scorer to count the number of retrieval calls in a trace.

    Returns:
        A scorer function.
    """
    from mlflow.genai import scorer

    @scorer
    def num_retrieval_calls(trace: Any) -> int:
        return sum(1 if span.span_type == "TOOL" else 0 for span in trace.data.spans)

    return num_retrieval_calls


def evaluate_agent(
    predict_fn: Any,
    data: list[dict[str, Any]],
    scorers: list[Any],
    experiment_name: str = "agent-evaluation",
) -> Any:
    """Run agent evaluation with MLflow.

    Args:
        predict_fn: Function that runs the agent and returns a trace.
        data: List of evaluation cases.
        scorers: List of scorers/judges to evaluate the agent.
        experiment_name: Name of the MLflow experiment.

    Returns:
        Evaluation results.
    """
    from mlflow.genai import evaluate

    setup_mlflow()
    mlflow.set_experiment(experiment_name)

    results = evaluate(
        data=data,
        predict_fn=predict_fn,
        scorers=scorers,
    )

    return results

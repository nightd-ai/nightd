import os
from unittest.mock import MagicMock, patch


def test_setup_mlflow():
    """Test that setup_mlflow configures the tracking URI correctly."""
    from daemon.agent import setup_mlflow

    test_uri = "http://localhost:5000"
    with patch.dict(os.environ, {"MLFLOW_TRACKING_URI": test_uri}):
        setup_mlflow()
        import mlflow

        assert mlflow.get_tracking_uri() == test_uri


def test_setup_mlflow_default_uri():
    """Test that setup_mlflow uses default URI when env var is not set."""
    from daemon.agent import setup_mlflow

    with patch.dict(os.environ, {}, clear=True):
        setup_mlflow()
        import mlflow

        assert mlflow.get_tracking_uri() == "http://localhost:5000"


def test_create_agent_options():
    """Test that create_agent_options creates a valid options object."""
    from claude_agent_sdk import ClaudeAgentOptions
    from daemon.agent import create_agent_options

    tools = []
    system_prompt = "You are a helpful assistant."

    options = create_agent_options(
        tools=tools,
        system_prompt=system_prompt,
        max_turns=5,
    )

    assert isinstance(options, ClaudeAgentOptions)
    assert options.system_prompt == system_prompt
    assert options.max_turns == 5


def test_create_dataset():
    """Test that create_dataset creates an MLflow dataset."""
    from daemon.agent import create_dataset, setup_mlflow
    import mlflow
    import time

    setup_mlflow()
    experiment_name = f"test-experiment-{int(time.time())}"
    experiment_id = mlflow.create_experiment(experiment_name)

    data = [
        {
            "inputs": {"query": "What is the capital of France?"},
            "expectations": {"expected_topics": "Paris"},
        },
    ]

    dataset = create_dataset(
        name="test-dataset",
        experiment_id=experiment_id,
        data=data,
        tags={"complexity": "basic"},
    )

    assert dataset is not None


def test_make_retrieval_judge():
    """Test that make_retrieval_judge creates a judge."""
    from daemon.agent import make_retrieval_judge

    judge = make_retrieval_judge()

    assert judge is not None
    assert judge.name == "retrieval_redundancy"


def test_make_comprehensiveness_judge():
    """Test that make_comprehensiveness_judge creates a judge."""
    from daemon.agent import make_comprehensiveness_judge

    judge = make_comprehensiveness_judge()

    assert judge is not None
    assert judge.name == "comprehensive"


def test_run_agent_with_timeout():
    """Test that run_agent_with_timeout handles the timeout correctly."""
    from daemon.agent import run_agent_with_timeout
    from claude_agent_sdk import ClaudeAgentOptions

    options = MagicMock(spec=ClaudeAgentOptions)

    with patch("asyncio.run") as mock_run:
        mock_run.return_value = "test response"
        result = run_agent_with_timeout("test query", options, timeout=60)

        assert result == "test response"
        mock_run.assert_called_once()

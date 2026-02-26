from unittest.mock import patch

from typer.testing import CliRunner

from daemon.cli import app

runner = CliRunner()


def test_start_default():
    """Test start command with default host and port."""
    with patch("daemon.cli.uvicorn.run") as mock_run:
        result = runner.invoke(app, ["start"])

        assert result.exit_code == 0
        mock_run.assert_called_once()
        call_args = mock_run.call_args
        assert call_args.kwargs["host"] == "127.0.0.1"
        assert call_args.kwargs["port"] == 8000


def test_start_custom_host():
    """Test start command with custom host."""
    with patch("daemon.cli.uvicorn.run") as mock_run:
        result = runner.invoke(app, ["start", "--host", "0.0.0.0"])

        assert result.exit_code == 0
        mock_run.assert_called_once()
        assert mock_run.call_args.kwargs["host"] == "0.0.0.0"


def test_start_custom_port():
    """Test start command with custom port."""
    with patch("daemon.cli.uvicorn.run") as mock_run:
        result = runner.invoke(app, ["start", "--port", "8080"])

        assert result.exit_code == 0
        mock_run.assert_called_once()
        assert mock_run.call_args.kwargs["port"] == 8080


def test_start_custom_host_and_port():
    """Test start command with custom host and port."""
    with patch("daemon.cli.uvicorn.run") as mock_run:
        result = runner.invoke(app, ["start", "-h", "0.0.0.0", "-p", "9000"])

        assert result.exit_code == 0
        mock_run.assert_called_once()
        assert mock_run.call_args.kwargs["host"] == "0.0.0.0"
        assert mock_run.call_args.kwargs["port"] == 9000

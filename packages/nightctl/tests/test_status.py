import httpx
from typer.testing import CliRunner

from nightctl.cli import app

runner = CliRunner()


def test_status_running(respx_mock):
    """Test status command when daemon is running."""
    respx_mock.get("http://localhost:8000/status").respond(200, json={"status": "OK"})

    result = runner.invoke(app, ["status"])

    assert result.exit_code == 0
    assert "Daemon is running: OK" in result.output


def test_status_stopped(respx_mock):
    """Test status command when daemon is not running."""
    respx_mock.get("http://localhost:8000/status").mock(
        side_effect=httpx.ConnectError("Connection refused")
    )

    result = runner.invoke(app, ["status"])

    assert result.exit_code == 0
    assert "Daemon is stopped" in result.output


def test_status_http_error(respx_mock):
    """Test status command when daemon returns an HTTP error."""
    respx_mock.get("http://localhost:8000/status").respond(
        500, text="Internal Server Error"
    )

    result = runner.invoke(app, ["status"])

    assert result.exit_code == 1
    assert "Error checking status" in result.output

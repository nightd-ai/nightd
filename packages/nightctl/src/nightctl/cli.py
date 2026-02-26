import httpx
import typer

app = typer.Typer(no_args_is_help=True)


@app.callback()
def main() -> None:
    """Nightctl CLI - command line interface for nightd."""
    pass


@app.command(name="status")
def status() -> None:
    """Check the status of the nightd daemon."""
    try:
        response = httpx.get("http://localhost:8000/status", timeout=5.0)
        response.raise_for_status()
        data = response.json()
        typer.echo(f"Daemon is running: {data.get('status', 'unknown')}")
    except httpx.ConnectError:
        typer.echo("Daemon is stopped")
    except httpx.HTTPError as e:
        typer.echo(f"Error checking status: {e}")
        raise typer.Exit(1)

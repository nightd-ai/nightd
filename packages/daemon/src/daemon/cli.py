import uvicorn
import typer

app = typer.Typer(no_args_is_help=True)


@app.callback()
def main() -> None:
    """Nightd daemon CLI."""
    pass


@app.command(name="start")
def start(
    host: str = typer.Option("127.0.0.1", "--host", "-h", help="Host to bind to"),
    port: int = typer.Option(8000, "--port", "-p", help="Port to bind to"),
) -> None:
    """Start the nightd daemon."""
    from daemon.api import app as fastapi_app

    uvicorn.run(fastapi_app, host=host, port=port)

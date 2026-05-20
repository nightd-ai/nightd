import typer

app = typer.Typer(no_args_is_help=True)


@app.callback()
def callback():
    pass


@app.command()
def status():
    print("Hello from nightctl!")


def main():
    app()


if __name__ == "__main__":
    main()

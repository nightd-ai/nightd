# nightd

A daemon to schedule autonomous coding agents.

## CLI Commands

- `nightd start [--host HOST] [--port PORT]` - Start the daemon (default: 127.0.0.1:8000)

## Development

### Prerequisites

- [jujutsu](https://www.jj-vcs.dev) for local version control
- [mise](https://mise.jdx.dev/) for development environment setup
- A coding agent

### Setup

1. Clone the repository:

   ```bash
   jj git clone git@github.com:nightd-ai/nightd.git
   cd nightd
   ```

2. Install tools and dependencies:

   ```bash
   mise trust
   mise install
   uv sync
   ```

3. Start the daemon:

   ```bash
   nightd start
   ```

### Tests

You can run the tests via:

```bash
pytest
```

### Documentation

To run the documentation locally:

```bash
zensical serve
```

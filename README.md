# nightd

A daemon to schedule autonomous coding agents.

## CLI Commands

- `nightd start [--host HOST] [--port PORT]` - Start the daemon (default: 127.0.0.1:8000)
- `nightctl status` - Check if the daemon is running

## Development

### Prerequisites

- [mise](https://mise.jdx.dev/) for development environment setup
- [Docker](https://www.docker.com/) to run Supabase locally
- [Rust](https://www.rust-lang.org/) (installed via mise)

### Setup

1. Install tools and dependencies:

   ```bash
   mise install
   cargo build
   ```

2. Start the daemon:

   ```bash
   cargo run --bin nightd -- start
   ```

### Tests

You can run the tests via:

```bash
cargo test
```

### Documentation

To run the documentation locally:

```bash
uvx zensical serve
```

# nightd

A daemon to schedule autonomous coding agents.

## Development

### Prerequisites

- [Rust](https://rust-lang.org/tools/install/)
- [uvx](https://docs.astral.sh/uv/getting-started/installation/) - for documentation

### Setup

1. Build the project:

   ```bash
   cargo build
   ```

2. Run the daemon:

   ```bash
   cargo run --bin nightd
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

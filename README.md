# nightd

A daemon to schedule autonomous coding agents.

## Development

### Prerequisites

- [mise](https://mise.jdx.dev/) for development environment setup

### Setup

1. Install tools and dependencies:

   ```bash
   mise trust
   mise install
   ```

2. Build the project:

   ```bash
   cargo build
   ```

3. Run the daemon:

   ```bash
   cargo run --bin nightd
   ```

### Tests

You can run the tests via:

```bash
cargo test
```

Or use mise:

```bash
mise run test
```

### Code Quality

Format code:

```bash
cargo fmt
```

Run clippy:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

### Documentation

To run the documentation locally:

```bash
uvx zensical serve
```

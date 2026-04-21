# Guidelines for agents

## Credentials

CRITICAL: NEVER try to read or write to `.env`. ALWAYS ask the user to modify it.

`.env` is automatically loaded by `mise`, do not add any library to load it.

## Dependencies

To add a new dependency add it to the crate with `cargo add -p`. Example:

```bash
cargo add -p nightctl clap --features=derive
```

ALWAYS check, if a dependency is already used by another crate. If a dependency is used by at least 2 crates. Move its version management to the Cargo workspace and reference it from each create. Example:

```toml
[dependencies]
clap.workspace = true
```

## Committing

### Pre-commit Checklist

Before committing changes on code, tests or dependencies do the following tasks:

- Format code - `cargo fmt`
- Run compile - `cargo check`
- Run linter - `cargo clippy -- -D warnings`
- Run tests - `cargo test`
- Fix all compile, linting and test errors and warnings

### Commit Signing

NEVER disable commit signing.

### Commit Messages

Use conventional commits for all commit messages.

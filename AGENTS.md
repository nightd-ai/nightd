# Guidelines for agents

## Credentials

CRITICAL: NEVER try to read or write to `.env`. ALWAYS ask the user to modify it.

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

Before committing changes on code, tests or dependencies run `mise run test`.

### Commit Signing

NEVER disable commit signing.

### Commit Messages

Use conventional commits for all commit messages.

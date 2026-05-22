# Guidelines for agents

This is a mono-repo with multiple apps written in Rust and Typescript. It uses Cargo and pnpm as a package manager and mise to manage the development environment and run common tasks.

## Credentials

CRITICAL: NEVER try to read or write to `.env`. ALWAYS ask the user to modify it.

`.env` is automatically loaded by `mise`, do not add any library to load it.

## Committing

### Pre-commit Checklist

Before committing changes on code, tests or dependencies do the following tasks:

- Format code - `mise run fmt`
- Run checks - `mise run ci`
- Fix all errors and warnings

### Commit Signing

NEVER disable commit signing.

### Commit Messages

Use conventional commits for all commit messages.

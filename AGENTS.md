# Guidelines for agents

## Credentials

CRITICAL: NEVER try to read or write to `.env`. ALWAYS ask the user to modify it.

`.env` is automatically loaded by `mise`, do not add any library to load it.

## Committing

### Pre-commit Checklist

Before committing changes on code, tests or dependencies do the following tasks:

- Format code - `mise run fmt`
- Run type checks - `mise run type-check`
- Run linter - `mise run lint`
- Fix all type and linting errors and warnings

### Commit Signing

NEVER disable commit signing.

### Commit Messages

Use conventional commits for all commit messages.

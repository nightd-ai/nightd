# Guidelines for agents

`nightd` is a daemon that runs on a server and polls Linear. It automatically picks tickets and applies the following steps:

1. Create workspace for the ticket
2. Launch OpenCode session for the ticket in the background
3. Track the linked pull request for the change
4. Launch additional sessions to react to review comments
5. Clean everything up after merging or closing the pull request

This project uses mise to manage the development environment and run common tasks. Do not add Makefiles.

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

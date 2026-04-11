# Guidelines for agents

## Credentials

CRITICAL: NEVER try to read or write to `.env`. ALWAYS ask the user to modify it.

### Github

Always enable automerge (--sqash) on non-draft pull requests.

### Pre-commit Checklist

Before committing changes on code, tests or dependencies run `mise run test`.

### Commit Signing

NEVER disable commit signing.

### Commit Messages

Use conventional commits for all commit messages.

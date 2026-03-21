# Guidelines for agents

## Tools

This project is managed by `uv` and is a workspace project with multiple packages.

ALWAYS use `uv add` and `uv remove` commands to manage dependencies.

## Credentials

CRITICAL: NEVER try to read or write to `.env`. ALWAYS ask the user to modify it.

## Git

### Pre-commit Checklist

Before committing changes on code, tests or dependencies run `mise run test`.

### Signing and Sign-off

ALWAYS commit changes with `git commit --signoff`. NEVER disable commit signing.

### Commit Messages

Use conventional commits for all commit messages.

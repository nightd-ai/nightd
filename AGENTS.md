# Guidelines for agents

## Tools

This project is managed by `uv` and is a workspace project with multiple packages.

ALWAYS use `uv add` and `uv remove` commands to manage dependencies.

## Credentials

CRITICAL: NEVER try to read or write to `.env`. ALWAYS ask the user to modify it.

## Jujutsu and Github

This project uses Jujutsu (jj) as version control system. To create a new pull request on Github do the following:

1. Update the repository from Github
2. Create a new change from `main`
3. Implement the changes
4. Push the changes to Github and create a pull request from it

Always enable automerge (--sqash) on non-draft pull requests.

### Pre-commit Checklist

Before committing changes on code, tests or dependencies run `mise run test`.

### Commit Signing

NEVER disable commit signing.

### Commit Messages

Use conventional commits for all commit messages.

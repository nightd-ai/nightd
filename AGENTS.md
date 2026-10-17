# Guidelines for agents

This is a mono-repo that uses `pnpm` as a package manager.

## Credentials

CRITICAL: NEVER try to read or write to `.env`. ALWAYS ask the user to modify it.

## Committing

### Pre-commit Checklist

Before committing changes on code, tests or dependencies do the following tasks:

- Format code - `pnpm run fmt`
- Type check - `pnpm -r typecheck`
- Lint - `pnpm run lint`
- Run tests - `pnpm -r test`
- Fix all errors and warnings

### Commit Messages

Use conventional commits for all commit messages.

### Developer Certificate of Origin

All commits MUST be signed off to certify the Developer Certificate of Origin. ALWAYS create commits with `git commit -s` to append a `Signed-off-by` trailer with your name and email.

```bash
git commit -s -m "feat: add something"
```

NEVER commit without a valid `Signed-off-by` line.

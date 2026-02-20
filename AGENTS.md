# AGENTS.md

This file provides guidelines for AI agents working in the nightd repository.

## Project Overview

nightd is a Python daemon for scheduling autonomous coding agents. It uses:

- Python 3.14+
- pytest for testing
- ruff for linting and formatting
- ty for type checking
- Supabase for database

## Build/Lint/Test Commands

```bash
# Run all tests
pytest

# Run a single test file
pytest tests/test_file.py

# Run a single test function
pytest tests/test_file.py::test_function_name

# Run tests matching a pattern
pytest -k test_pattern

# Format code
ruff format .

# Fix linting issues
ruff check --fix .

# Check linting without fixing
ruff check .

# Type checking
ty check .
```

## Code Style Guidelines

### Imports

- Use absolute imports over relative imports
- Group imports: stdlib, third-party, local
- Sort imports with ruff (isort rules enabled)

### Formatting

- Line length: 88 characters (ruff default)
- Use double quotes for strings
- Trailing commas in multi-line collections
- Run `ruff format .` before committing

### Types

- Use type hints for all function signatures
- Use `ty` for type checking (not mypy/pyright)
- Prefer `|` over `typing.Union` for unions (Python 3.10+)
- Use `None` instead of `Optional[T]` for nullable types

### Naming Conventions

- `snake_case` for functions, variables, modules
- `PascalCase` for classes
- `UPPER_CASE` for constants
- Private functions prefix with `_`

### Error Handling

- Use specific exceptions, avoid bare `except:`
- Prefer `try/except` over `if` checks for expected errors
- Log exceptions with context before re-raising

### Testing

- Test files: `tests/test_*.py` or `*_test.py`
- Use descriptive test function names
- Use pytest fixtures for setup/teardown
- Mock external dependencies (database, API calls)

## Project Structure

```
nightd/
├── main.py              # Entry point
├── pyproject.toml       # Project config
├── mise.toml            # Task definitions
├── supabase/            # Database migrations/config
│   ├── config.toml
│   └── snippets/
├── tests/               # Test files (create as needed)
└── .env                 # Environment variables (not committed)
```

## Pre-commit Checklist

Before committing changes:

1. Run `mise run format` - code is formatted
2. Run `mise run linter:fix` - no lint errors
3. Run `mise run type-check` - type checks passA daemon to schedule autonomous coding agents
4. Run `mise run test` - all tests pass

## Notes

- Python version: 3.14+
- Virtual environment: `.venv/` (managed by uv)

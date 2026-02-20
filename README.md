# nightd

A daemon to schedule autonomous coding agents.

## Development

### Prerequisites

- [mise](https://mise.jdx.dev/) for development environment setup
- [Docker](https://www.docker.com/) to run Supabase locally

### Setup

1. Install tools and dependencies:

   ```bash
   mise install
   uv sync
   ```

2. Start Supabase locally:

   ```bash
   supabase start
   ```

3. Add the following environment variables to `.env`:

   ```bash
   DATABASE_URL=postgresql://postgres@127.0.0.1:54322/postgres
   DATABASE_PASSWORD=<password>
   ```

   The values can be obtained by running `supabase status`.

### Tasks

Run development tasks with `mise run`:

- `mise run format` - Run ruff formatter
- `mise run linter:fix` - Fix linting errors with ruff
- `mise run type-check` - Do type checking with ty
- `mise run test` - Run tests with pytest
- `mise run docs` - Run documentation locally

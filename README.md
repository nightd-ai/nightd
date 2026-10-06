# nightd

A daemon to schedule autonomous coding agents.

## Documentation

The documentation site is built with [Zensical](https://zensical.org).

### Prerequisites

- [Node.js](https://nodejs.org/) (LTS)
- [pnpm](https://pnpm.io/)

### Setup

```bash
pnpm install
```

### Development

```bash
pnpm dev
```

### Checks

```bash
pnpm run fmt:check
pnpm run lint
pnpm -r typecheck
pnpm -r test
```

### Building the docs

```bash
uvx zensical build --clean
```

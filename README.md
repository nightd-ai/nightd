# nightd

A daemon to schedule autonomous coding agents.

### Prerequisites

- [Node.js](https://nodejs.org)
- [pnpm](https://pnpm.io)

### Setup

```bash
pnpm install
```

### Development

Run the control plane:

```bash
pnpm --filter nightd dev
```

Run the data plane:

```bash
pnpm --filter nightlet dev
```

### Run the docs

```bash
uvx zensical serve
```

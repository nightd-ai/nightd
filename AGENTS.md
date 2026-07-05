# Guidelines for agents

## What is nightd

Nightd is a hosting service for Omnigent, an open source AI agent framework and meta-harness that orchestrates Claude Code, Codex, Cursor, Pi, and custom agents developed mainly by Databricks. Omnigent follows a control plane / data plane architecture with a server that acts as control plane and a runner that acts as the data plane. It also provides a UI (web, desktop app, mobile app) and a CLI. Omnigent itself is single-tenant.

Nightd provides Omnigent hosting by acting as a multi-tenant proxy in front of the Omnigent server's API. Each tenant gets its own Omnigent server under a dedicated subdomain. Apps, CLI and runners can connect to the server through nightd's proxy.

### Authentication

Authentication is handled by nightd in a multi-tenant way, where a user can sign-in and then access a server. A user can have access to one or multiple servers.

Nightd uses Omnigent's header authentication. It reads a token from cookie or bearer token, verifies the token and forwards the user's email via `X-Forward-Email` header to Omnigent.

### Serverless Hosting

Nightd targets indie hackers and small teams. Therefore, we assume many servers with low traffic and data per server. To make the hosting efficient a serverless approach must be implemented.

#### Scale to Zero

Nightd acts as a proxy in front of Omnigent servers and must start a server ad-hoc, when it is not running. It tracks the life cycle of each server and scales it to zero, when a server is idle.

#### Database

Each Omnigent server requires its own database. Nightd uses SQLite as a database for Omnigent to put each server's state in a single file.

SQLite forces nightd to run not more than one instance per Omnigent server.

## Architecture

This is a mono-repo with multiple apps written in Go. It uses mise to setup the development environment and implement common tasks.

### Apps

There are three apps in this project that are build as separate binaries and shipped as separate container images.

- gateway: handles authentication and forwards requests to the Omnigent server of the current tenant
- api: control plane that manages the life cycle of Omnigent servers and provides the correct routing to the gateway
- node: data plane that launches an Omnigent server inside a Linux container on a concrete node

The api app is the only one that uses a Postgres database and provides an API. The gateway acts only as a proxy for Omnigent API. The node connects itself to the api to receive instructions.

The API provided uses gRPC.

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

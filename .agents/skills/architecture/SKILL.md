---
name: architecture
description: Use this skill, when you plan the implementation of a feature and you need to know more about the project's architecture and design. Use it, when working on CLI, API, database or configuration design.
---

`nightd` is a tool to schedule agents asynchronously in the background like subagents. It is build on top of the Agent Client Protocol (ACP).

It is build of 2 separate binaries:

- `nightd` - the daemon to schedule and execute agent sessions via ACP
- `nightctl` - the CLI to control the daemon

Usually, the daemon will be started, when the user logs in (e.g. via systemd on Linux via `systemctl enable --user --now nightd.service`).

`nightd` stays as close to ACP as possible and reuses the terminology of the protocol as an internal domain language (e.g. prompt, session, turn).

Checkout the following references for more details:

- @references/api.md - API design guidelines
- @references/cli.md - CLI design guidelines
- @references/configuration.md - Configuration guidelines
- @references/database.md - SQLite database guidelines
- @references/testing.md - Testing guidelines

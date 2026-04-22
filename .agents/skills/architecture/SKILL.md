---
name: architecture
description: Use this skill, when you plan the implementation of a feature and you need to know more about the project's architecture and design. Use it, when working on CLI, API, database or configuration design.
---

`nightd` is a tool to schedule agents asynchronously in the background like subagents. It is build on top of the Agent Client Protocol (ACP).

### CLI and `nightd` Daemon

The general setup consists of 2 parts, which are distributed as separate binaries:

- `nightd` - the daemon to schedule and execute agent sessions via ACP
- `nightctl` - the CLI to control the daemon

Usually, the daemon will be started, when the user logs in (e.g. via systemd on Linux via `systemctl enable --user --now nightd.service`).

### ACP

`nightd` stays as close to ACP as possible and reuses the terminology of the protocol as an internal domain language (e.g. prompt, session, turn). Also the API uses payloads aligned with ACP.

### Daemon API

The daemon exposes a HTTP server (usually on localhost) to provide an API. The API can be used to query or modify the daemon's state.

### CLI

The CLI provides multiple commands that can simply invoke the daemon's API or in some cases do additional things (e.g. update the configuration directly). Commands are aligned with ACP terms and, if they access the API the following convention for subcommands is used:

- `nightctl session new` - `POST` on sessions endpoint to create a session
- `nightctl session ls` - `GET` on sessions endpoint to list all sessions
- `nightctl session show` - `GET` on session endpoint to show details of a specific session
- `nightctl session rm` - `DELETE` on session endpoint to delete a specific session

### Configuration

The main configuration is in `$NIGHTD_CONFIG_DIR/config.toml`. The default value is resolved from the dirs crate.

The configuration can be edited directly via a text editor or via `ngightctl` CLI, which provides a `config` command with `set`, `unset` and `get` subcommands. After changing the configuration, the daemon must be reloaded.

### SQLite Database

`nightd` daemon stores its state in a SQLite database, which is stored in the app data directory of the user. It contains the sessions that are scheduled, its status and the updates and result from the agent.

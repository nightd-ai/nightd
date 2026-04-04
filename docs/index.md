---
title: Welcome to nightd
---

`nightd` is like Docker for AI agent tasks. Instead of working interactively with your coding agent, schedule coding tasks in the background and review the created pull requests.

!!! warning "Under Active Development"

    `nightd` is currently being built and is not ready for use. You can try it out by checking out the source code. We will provide easy to install releases later. Early adopters welcome — help shape the future!

## How it works

`nightd` runs as a local daemon that manages agent tasks through [ACP](https://agentclientprotocol.com) in [devcontainers](https://containers.dev), supporting [mise](https://mise.jdx.dev) as an easy alternative to writing Dockerfiles. Each task runs in its own [Jujutsu](https://www.jj-vcs.dev) workspace, isolating changes and avoiding conflicts with other sessions. Your agents create branches and submit pull requests on your behalf — all without cloud dependencies.

## Features

- **Background Tasks** — Schedule tasks in `nightd` and do something else in the meantime
- **Agent- and Provider-Agnostic** — Use your existing agentic setup with support for OpenCode, Claude Code and others
- **Local-first** — Works fully on your laptop without any subscriptions

## Join the Community

We're building nightd in the open and would love your input:

- **⭐ Star** us on [GitHub](https://github.com/nightd-ai/nightd) to show support
- **🦋 Follow** [@nightd.ai](https://bsky.app/profile/nightd.ai) on Bluesky for updates
- **💬 Join** our [Discord](https://discord.gg/XzwUyUQZ9r) to share feedback and ideas

---
title: 001 - ACP Native Architecture
---

## Status

Proposed

## Context

The goal of `nightd` is to allow the daemon to schedule coding agents in the background and let them work autonomously on tasks. A crucial requirement is to support as many models and model providers as possible, because every engineering organization or indie hacker has their own subscriptions and policies regarding model vendors and providers. Some are subscribed to Cursor, some only allow usage of Claude Code via AWS Bedrock to enforce hosting of models in the EU.

There are two approaches to achieve a model and vendor agnostic design:

1. **Use a single vendor agnostic agent** in `nightd` via their SDK (e.g., Pi or OpenCode)
2. **Support different agents** as a back-end in `nightd`

OpenClaw is an example of the first approach, using Pi via their SDK as an agent under the hood. However, this approach presents challenges for `nightd`. Software engineers currently use agents mostly interactively, and have started to develop skills, tools, and workflows with subagents for themselves or shared within their organizations. Different engineers might choose different agents, similar to how they have chosen different editors or IDEs.

Since `nightd` is local-first software designed to run on the engineer's laptop, a smooth transition from interactive use to autonomous agents can only be guaranteed if the agent setup stays as consistent as possible between both modes of operation. This enables good co-existence and allows self-paced adoption following a progressive enhancement principle.

This leads to the second option: supporting many agents. Most coding agents provide SDKs for programmatic use, which could be used to integrate them into `nightd`. We could internally define an API for interaction with agents and then implement agent-specific adapters. However, `nightd` is written in Rust and most SDKs are for TypeScript or Python. Instead of directly accessing the SDK, an adapter would require some IPC protocol to bridge between different programming languages. Alternatively, embedding the Deno JavaScript runtime could work for agents written in TypeScript, but agents written in Python or other languages would still require IPC.

The **Agent Client Protocol (ACP)** is a standard protocol designed for editors and IDEs to embed agents. It standardizes communication between code editors/IDEs and coding agents and is suitable for both local and remote scenarios.

### ACP Architecture

ACP is built on several key architectural principles:

1. **MCP-friendly**: The protocol is built on JSON-RPC 2.0, and re-uses MCP (Model Context Protocol) types where possible so that integrators don't need to build yet-another representation for common data types.
2. **UX-first**: It is designed to solve the UX challenges of interacting with AI agents; ensuring there's enough flexibility to render clearly the agent's intent, but is no more abstract than it needs to be.
3. **Trusted**: ACP works when you're using a code editor to talk to a model you trust. You still have controls over the agent's tool calls, but the code editor gives the agent access to local files and MCP servers.

### Communication Model

The protocol follows the JSON-RPC 2.0 specification with two types of messages:

- **Methods**: Request-response pairs that expect a result or error
- **Notifications**: One-way messages that don't expect a response

In local scenarios, the client (e.g., `nightd`) boots the agent sub-process on demand, and all communication happens over stdin/stdout. Each connection can support several concurrent sessions, enabling multiple independent tasks to run simultaneously.

ACP makes heavy use of JSON-RPC notifications to allow the agent to stream updates to the client in real-time. It also uses JSON-RPC's bidirectional requests to allow the agent to make requests of the client—for example, to request permissions for a tool call.

### Key ACP Concepts

- **Sessions**: Conversational contexts that maintain state between prompts
- **Prompt Turns**: The core conversation flow where the client sends user messages and the agent responds
- **Content Blocks**: Structured message content including agent messages, user messages, thoughts, tool calls, and plans
- **Tool Calls**: How agents report tool execution with permission requests
- **Slash Commands**: Agents can advertise available slash commands to clients
- **Terminals**: Support for executing and managing terminal commands
- **File System**: Client-side filesystem access methods

Some agents support ACP natively (e.g., OpenCode, crow-cli, Gemini CLI), while the community has built adapters using their SDKs for others (e.g., Claude Agent by Zed, Codex, Pi via pi-acp).

### The ACP Registry

The ACP Registry provides a standardized way to discover and install ACP-compatible agents. The registry contains metadata about agents including their distribution information for automatic installation. Clients can fetch the registry programmatically at `https://cdn.agentclientprotocol.com/registry/v1/latest/registry.json`.

### The Rust SDK

The `agent-client-protocol` Rust crate provides implementations of both sides of the Agent Client Protocol. To build with ACP, we implement either the `Agent` trait or the `Client` trait to define the interaction with the ACP counterpart. This crate powers the integration with external agents in the Zed editor.

## Decision

We will build `nightd` ACP native to leverage the existing ecosystem, including:

- The [Rust SDK](https://crates.io/crates/agent-client-protocol) for ACP
- Existing community adapters for popular agents
- The [ACP Registry](https://agentclientprotocol.com/get-started/registry) for agent discovery

### What "ACP Native" Means

Being "ACP native" goes beyond simply using ACP as a communication protocol. It means establishing ACP as the foundational domain language for agents within `nightd`:

- **Terminology alignment**: Terms like sessions, prompts, content blocks, tool calls, and message types will align with ACP definitions as much as possible
- **Conceptual model**: The internal architecture will mirror ACP concepts:
  - Tasks map to ACP sessions (isolated conversational contexts)
  - Background jobs follow the prompt turn lifecycle
  - Updates use ACP content block types (message chunks, tool calls, plans)
- **Protocol fidelity**: We will use ACP not just as a transport mechanism, but as the canonical way to express agent interactions:
  - Use JSON-RPC 2.0 as the underlying transport
  - Support concurrent sessions for parallel task execution
  - Handle bidirectional communication (client can interrupt agents, agents can request permissions)
- **Ecosystem integration**: Use the ACP Registry for agent discovery and installation

## Consequences

### Positive

- **Broad agent support**: Immediate compatibility with agents that support ACP natively (OpenCode, Gemini CLI, crow-cli) and through community adapters (Claude Agent by Zed, Codex via Zed's adapter, Pi via pi-acp)
- **Ecosystem leverage**: Can use the existing Rust SDK and community adapters rather than building custom IPC bridges for each agent
- **Consistency**: Engineers can use the same agent configuration interactively and in background tasks
- **Future-proof**: As the ACP ecosystem grows, `nightd` automatically gains support for new agents via the ACP Registry
- **Reduced maintenance**: No need to maintain multiple language-specific SDK integrations
- **Session management**: ACP's session model naturally supports the isolation we need between concurrent background tasks
- **Real-time updates**: JSON-RPC notifications enable streaming progress updates from background agents

### Negative

- **Dependency on ACP ecosystem**: If ACP adoption stalls or the protocol changes significantly, `nightd` may need adaptation
- **Potential limitations**: ACP may not expose all features of specific agents, requiring workarounds or extensions via the protocol's extensibility mechanisms (`_meta` fields, custom methods prefixed with `_`)
- **Learning curve**: Contributors need to understand ACP concepts in addition to `nightd` internals

### Neutral

- **MCP relationship**: ACP builds on the Model Context Protocol (MCP). We will monitor how these protocols evolve and may need to adapt our architecture as the standards mature.
- **Remote agent support**: Full support for remote agents is a work in progress in the ACP ecosystem. While `nightd` is designed for local-first operation, future remote agent support could extend our capabilities.

## References

- [Agent Client Protocol Documentation](https://agentclientprotocol.com/)
- [ACP Rust SDK](https://docs.rs/agent-client-protocol/latest/agent_client_protocol/)
- [ACP Registry](https://agentclientprotocol.com/get-started/registry)
- [ACP Architecture Overview](https://agentclientprotocol.com/get-started/architecture)

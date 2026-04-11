---
name: agentclientprotocol
description: Use this skill, when you are asked about the Agent Client Protocol (ACP). Use it, when you work on code related to ACP (e.g. when using the ACP SDK).
---

The **Agent Client Protocol (ACP)** is a standard protocol designed for editors and IDEs to embed agents. It standardizes communication between code editors/IDEs and coding agents and is suitable for both local and remote scenarios.

### ACP Architecture

ACP is built on several key architectural principles:

1. **MCP-friendly**: The protocol is built on JSON-RPC 2.0, and re-uses MCP (Model Context Protocol) types where possible so that integrators don't need to build yet-another representation for common data types.
2. **UX-first**: It is designed to solve the UX challenges of interacting with AI agents; ensuring there's enough flexibility to render clearly the agent's intent, but is no more abstract than it needs to be.
3. **Trusted**: ACP works when you're using a code editor to talk to a model you trust. You still have controls over the agent's tool calls, but the code editor gives the agent access to local files and MCP servers.

### References

- @references/documentation.md - protocol documentation
- @references/rust-sdk-v1.md - Rust SDK documentation

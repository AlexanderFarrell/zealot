# Zealot Documentation

---

## Getting Started

- [Overview](./overview.md) — What Zealot is, core use cases, and how it compares to adjacent tools
- [Quickstart](./quickstart.md) — Install Zealot, create your first workspace, and use it for real planning
- [Deployment and Operations](./deployment.md) — Run Zealot on a server: configuration, TLS, backups, security, and upgrades
- [Building & Running](./building.md) — Build the backend, web app, mobile apps, and Docker stack
- [Migrations](./migrations.md) — Database schema migration system

## Features

- [ZealotScript](./zealotscript/README.md) — Zealot's markup language: syntax reference
  - [Emoji](./zealotscript/emoji.md) — Emoji shortcode syntax and reference
  - [Math](./zealotscript/math.md) — LaTeX/KaTeX math expression support
- [Rules Engine](./rules-engine.md) — Lua-based automation: triggers, API reference, cookbook

## Developer Guides

- [Architecture](./architecture.md) — Current system architecture (Rust + TypeScript)
- [Analysis Tooling](./analysis-tooling.md) — Local code analysis setup

## Integrations

- [MCP System Prompt](./mcp-system-prompt.md) — Zealot's MCP tools for LLM agents
- [HTTP API](./api.md) — REST API overview

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for build, test, and PR instructions.

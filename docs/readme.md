# Zealot Documentation

---

## Getting Started

- [Overview](./overview.md) — What Zealot is, core use cases, creative use cases, and how it compares to adjacent tools
- [Quickstart](./quickstart.md) — Install Zealot, create your first workspace, and use it for real planning
- [Deployment and Operations](./deployment.md) — Run Zealot on a server: configuration, TLS, backups, security, and upgrades
- [Troubleshooting and Recovery](./troubleshooting.md) — Diagnose startup, login, migration, and database problems; restore from backup; data-loss decision tree
- [Building & Running](./building.md) — Build the backend, web app, mobile apps, and Docker stack
- [Migrations](./migrations.md) — Database schema migration system

## Reference

- [Data Model](./data-model.md) — canonical reference for items, types, attributes, links, repeats, comments, rules, and ZealotScript
- [Glossary](./glossary.md) — definitions of all core Zealot terms

## Features

- [ZealotScript](./zealotscript/README.md) — Zealot's markup language: syntax reference
  - [Emoji](./zealotscript/emoji.md) — Emoji shortcode syntax and reference
  - [Math](./zealotscript/math.md) — LaTeX/KaTeX math expression support
- [Rules Engine](./rules-engine.md) — Lua-based automation: triggers, API reference, cookbook

## Developer Guides

- [Architecture](./architecture.md) — Current system architecture (Rust + TypeScript)
- [Analysis Tooling](./analysis-tooling.md) — Local code analysis setup

## Terminal Clients

- [Terminal Guide](./terminal-guide.md) — full user guide for the CLI and TUI: login, profiles, daily planning, item workflows, search, habits, comments, rules, media, scripting, keybindings, and troubleshooting
- [CLI](./cli.md) — the `zealot` command: full wiki/planner/habit control from a shell, JSON output for scripting, `$EDITOR` round-trips, shell completions
- [TUI](./tui.md) — `zealot-tui`, a full-screen keyboard-driven workstation: Today dashboard, item browser, live search, habit grid, rules

## Integrations

- [MCP Guide](./mcp.md) — Connect Claude, Codex, or any MCP client to Zealot: installation, tool reference, workflows, and safe operating rules
- [MCP System Prompt](./mcp-system-prompt.md) — Agent-facing system prompt describing Zealot's data model and tool behavior
- [HTTP API](./http-api.md) — REST API reference
- [Client Crate](./client-crate.md) — `zealot-client`, the shared Rust HTTP client used by the CLI, TUI, and MCP server

## Contributing

- [CONTRIBUTING.md](../CONTRIBUTING.md) — documentation style guide, change policy, PR checklist, command validation, and screenshot expectations; code build and test notes

# Zealot

Zealot is a self-hosted personal wiki and planner. Everything — notes, tasks, goals, projects, habits — is an **item**: a titled document with a freeform body, user-defined attributes, and a place on the planner. Your data lives in a database on your own machine or server. No cloud subscription, no vendor lock-in.

## Why Zealot?

Most personal productivity tools make you choose: a task manager *or* a note-taking app *or* a journal *or* a habit tracker. Zealot unifies them. A task and a wiki page are the same kind of thing. A goal can have a body with your strategy, child tasks with due dates, and a habit to track daily progress — all linked together and all appearing on the same planner.

Because Zealot is self-hosted and API-first, it is also programmable. You can automate workflows with server-side Lua scripts, query and update your data over HTTP from any language, or connect an AI agent (Claude, Codex CLI, LibreChat) to your wiki and planner via the built-in MCP server.

**Good fit if you:**
- Want one place for notes, tasks, and planning — not three separate apps
- Care about data ownership and want nothing in someone else's cloud
- Are comfortable with Git and Docker, and want a system you can extend
- Want automation that runs on a schedule or reacts to your data changing

**Not a good fit if you:**
- Need polished mobile apps or zero-setup cloud access
- Share a workspace with a team (Zealot is single-user first)
- Want drag-and-drop building without any configuration

See [how Zealot compares to Notion, Obsidian, Todoist, and Org Mode](./docs/overview.md#how-zealot-compares).

## Getting Started

Zealot builds and runs via Docker Compose. The first build takes a few minutes; subsequent starts are fast.

```bash
git clone https://github.com/your-org/zealot.git
cd zealot
npm run docker
```

The web UI will be available at `http://localhost:8085`. The API runs on port 8456.

For a step-by-step walkthrough — creating your first types, attributes, and planner items — see the [Quickstart](./docs/quickstart.md).

## Use Cases

- [Planning and task tracking](./docs/overview.md#1-planning) — daily, weekly, monthly planner with scheduled items
- [Personal wiki](./docs/overview.md#2-personal-wiki) — linked documents with wiki links, callout blocks, and math
- [Goals and habit tracking](./docs/overview.md#4-recurring-work) — repeat tracker, cron automation, and goal cascades
- [Automation](./docs/overview.md#6-automation) — Lua rules engine for cron and event-driven workflows
- [AI agent integration](./docs/mcp.md) — connect Claude Desktop or any MCP client to your wiki and planner
- [Creative use cases](./docs/overview.md#creative-use-cases) — reading tracker, weekly reviews, research base, AI journaling

## Documentation

Full documentation is at [docs/readme.md](./docs/readme.md).

Key starting points:

| | |
|---|---|
| [Overview](./docs/overview.md) | What Zealot is and how it compares to alternatives |
| [Quickstart](./docs/quickstart.md) | Install, configure, and use Zealot in ~10 minutes |
| [Data Model](./docs/data-model.md) | Items, types, attributes, links, rules — the full reference |
| [Rules Engine](./docs/rules-engine.md) | Lua automation: triggers, API, and a cookbook of examples |
| [MCP Guide](./docs/mcp.md) | Connect an AI agent to Zealot |
| [HTTP API](./docs/http-api.md) | REST API reference for scripts and integrations |

## License

TODO

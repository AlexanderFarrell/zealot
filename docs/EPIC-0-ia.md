# Epic 0 — Documentation Information Architecture

This document is the output of the documentation audit ticket. It defines the inventory, audience map, target tree, and prioritised rewrite order for the rest of the Epic 0 documentation pass.

---

## 1. Inventory

### Root / Structural READMEs

| File | Audience | Status | Notes |
|---|---|---|---|
| `readme.md` | Evaluators, Contributors | Stub | Three `TODO` sections; no real content |
| `CLAUDE.md` | AI agents (Claude Code) | Good | Rewrite methodology and conventions; not public docs |
| `docs/readme.md` | All | Stub | Single line; no navigation |
| `crates/readme.md` | Backend developers | Good | Clean crate-layer summary |
| `apps/readme.md` | All | Broken | File body is duplicated — full content appears twice |
| `test/readme.md` | Contributors | Good | Complete E2E smoke-test guide |

### `docs/` Top-Level

| File | Audience | Status | Notes |
|---|---|---|---|
| `docs/building.md` | Developers, Deployers | Good | Rust, Web, Mobile (Tauri), Docker |
| `docs/migrations.md` | Backend developers | Good | SQLite/Postgres migration system |
| `docs/analysis-tooling.md` | Developers | Good | Local analysis setup, SonarQube prep |
| `docs/rules-engine.md` | End users | Good | Lua scripting API with cookbook |
| `docs/mcp-system-prompt.md` | MCP clients (LLMs) | Good | Operational system prompt for AI agents |
| `docs/right-click-menu.md` | — | Planning artifact | Design planning doc; not a user or developer guide |
| `docs/zealotscript-feature-ideas.md` | — | Planning artifact | Feature roadmap; not a user or developer guide |

### `docs/zealotscript/`

| File | Audience | Status | Notes |
|---|---|---|---|
| `docs/zealotscript/README.md` | End users | Good | Markup syntax reference |
| `docs/zealotscript/emoji.md` | End users | Good | Emoji shortcode syntax and reference |
| `docs/zealotscript/math.md` | End users | Good | LaTeX/KaTeX math expression support |

### `docs/legacy/` (9 files)

All nine files describe the **old Go/Fiber backend with PostgreSQL** (`zealotd/`, `client/`). The current stack is Rust (Axum) with SQLite/Postgres and a TypeScript Web Components frontend. Every file in this directory is stale.

| File | Notes |
|---|---|
| `legacy/index.md` | Index for the old Go docs |
| `legacy/architecture.md` | Old PostgreSQL + Go architecture |
| `legacy/architecture2.md` | Second old architecture doc |
| `legacy/backend.md` | Go API server structure |
| `legacy/frontend.md` | Old vanilla-TS client structure |
| `legacy/database.md` | PostgreSQL schema overview |
| `legacy/development.md` | Old build and run workflow |
| `legacy/todo.md` | Stale task list |
| `legacy/zealotscript_spec.md` | Superseded by `docs/zealotscript/README.md` |

### `docs/ai-tasks/` (Engineering Planning Record)

These are AI task inputs and architecture design documents — the project's engineering planning record. They are not end-user or developer documentation and should remain in `docs/ai-tasks/` unchanged.

| Subdirectory / File | Contents |
|---|---|
| `PLAN.md` – `PLAN8.md` | Cross-platform architecture decisions |
| `client_api.md` | Frontend API audit and discrepancy list |
| `comment.md` | Comment feature implementation plan |
| `rewrite/TASK-001` – `TASK-037` | UI rewrite implementation tickets |
| `rewrite/TASK-LUA-001` – `TASK-LUA-012` | Lua engine backend tickets |
| `rewrite/TASK-LUA-FE-001` – `TASK-LUA-FE-004` | Lua engine frontend tickets |
| `mcp/MCP-001` – `MCP-007` | MCP server implementation tickets |
| `analysis/ANALYSIS-001` – `ANALYSIS-006` | Analysis feature tickets |
| `zealotscript/TASK-021` – `TASK-034` | ZealotScript editor feature tickets |
| `rules-engine/` | Lua rules engine architecture and use-case docs |

---

## 2. Audience Map

| Audience | Primary Need | Current Coverage | Gap |
|---|---|---|---|
| **Evaluators** | What is Zealot? Why use it? Quick start. | `readme.md` — stub only | Full rewrite of `readme.md` |
| **End Users** | Learn and use ZealotScript, rules, planner | `docs/zealotscript/`, `docs/rules-engine.md` | Good; minor polish only |
| **Deployers** (self-host) | Run the stack; configure; upgrade | `docs/building.md` | Good; a dedicated deployment guide could expand on config |
| **Backend Developers** | Crate architecture, add features, migrations | `crates/readme.md`, `docs/migrations.md` | Missing current-stack architecture overview |
| **Frontend Developers** | Package layout, component pattern, API layer | `CLAUDE.md` (rewrite instructions) | Missing a permanent `packages/` README |
| **API Clients** | Consume the HTTP REST API | Nothing — source code only | Needs `docs/api.md` or OpenAPI |
| **MCP Clients** (LLM agents) | Available tools, patterns, Lua API | `docs/mcp-system-prompt.md` | Good |
| **Contributors** | Build, test, submit PRs | `docs/building.md`, `test/readme.md` | Missing `CONTRIBUTING.md` to link them together |
| **Future Maintainers** | Architecture decisions, crate roles, why-not patterns | Nothing current (`docs/legacy/architecture.md` is stale Go) | New `docs/architecture.md` needed |

---

## 3. Target Documentation Tree

```
readme.md                           ← Rewrite (P1): evaluator overview + quick-start
CONTRIBUTING.md                     ← New (P2): contributor guide linking build + test
CLAUDE.md                           ← Keep: AI rewrite conventions (not public)

docs/
  readme.md                         ← Rewrite (P1): navigation index for all /docs

  # Developer — Building & Running
  building.md                       ← Keep (minor polish)
  migrations.md                     ← Keep
  architecture.md                   ← New (P1): current Rust+TS architecture

  # End User — Features
  zealotscript/
    README.md                       ← Keep
    emoji.md                        ← Keep
    math.md                         ← Keep
  rules-engine.md                   ← Keep (minor polish)

  # Integrations
  mcp-system-prompt.md              ← Keep (operational LLM system prompt)
  api.md                            ← New (P3): HTTP REST API overview

  # Internal Planning (not published as guides)
  analysis-tooling.md               ← Keep
  right-click-menu.md               ← Add planning banner (P2)
  zealotscript-feature-ideas.md     ← Add planning banner (P2)
  EPIC-0-ia.md                      ← This file

  legacy/                           ← Delete all 9 files (P1)

  ai-tasks/                         ← Keep as-is (engineering planning record)

crates/readme.md                    ← Keep
apps/readme.md                      ← Fix duplicate content (P1)
packages/readme.md                  ← New (P3): TypeScript package layout guide
test/readme.md                      ← Keep
```

---

## 4. Stale, Duplicate, or Misleading Files

| File | Problem | Recommended Action |
|---|---|---|
| `docs/legacy/` (all 9) | Describes the old Go/Postgres stack; entirely superseded by the current Rust/TypeScript codebase | **Delete** |
| `apps/readme.md` | Full content is duplicated — appears twice in the same file | **Fix** — remove the second copy |
| `readme.md` | Three `TODO` blocks; nothing useful for evaluators or contributors | **Rewrite** |
| `docs/readme.md` | Single line with no navigation | **Rewrite** |
| `docs/right-click-menu.md` | Design planning artifact surfacing as documentation | **Add planning banner** or move to `docs/ai-tasks/` |
| `docs/zealotscript-feature-ideas.md` | Roadmap artifact surfacing as documentation | **Add planning banner** or move to `docs/ai-tasks/` |

---

## 5. What Is Missing

| Missing Doc | Needed By | Priority |
|---|---|---|
| Evaluator overview in `readme.md` | Evaluators | P1 |
| Navigation index in `docs/readme.md` | All | P1 |
| Current-stack architecture (`docs/architecture.md`) | Developers, Maintainers | P1 |
| `CONTRIBUTING.md` | Contributors | P2 |
| HTTP API reference (`docs/api.md`) | API clients | P3 |
| `packages/readme.md` (TypeScript package layout) | Frontend developers | P3 |

---

## 6. Prioritised Rewrite Order for Epic 0

| # | File | Why |
|---|---|---|
| 1 | **Rewrite `readme.md`** | First thing evaluators and contributors see. Currently all TODOs. |
| 2 | **Rewrite `docs/readme.md`** | Entry point for all documentation navigation. Currently 1 line. |
| 3 | **Delete `docs/legacy/`** | Actively misleading — describes a stack that no longer exists. |
| 4 | **Fix `apps/readme.md` duplicate** | Quick fix; file is broken. |
| 5 | **New `docs/architecture.md`** | No current-stack architecture doc exists. Essential for maintainers and new contributors. |
| 6 | **New `CONTRIBUTING.md`** | Links `docs/building.md` + `test/readme.md`; needed for open-source posture. |
| 7 | **Add planning banners** to `right-click-menu.md` and `zealotscript-feature-ideas.md` | Prevents confusion about their status. |
| 8 | **New `docs/api.md`** | HTTP API surface is undocumented outside source code. |
| 9 | **New `packages/readme.md`** | Helps frontend contributors orient without reading `CLAUDE.md`. |

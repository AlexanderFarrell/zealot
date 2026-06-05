# Zealot — Developer Guide

This guide covers the repo layout, local dev setup, architecture, testing, a first-contribution walkthrough, and troubleshooting. The audience is a developer who wants to maintain or extend Zealot.

---

## Table of Contents

1. [Repo Layout](#1-repo-layout)
2. [Local Development](#2-local-development)
3. [Architecture](#3-architecture)
4. [Testing](#4-testing)
5. [First Contribution](#5-first-contribution)
6. [Troubleshooting](#6-troubleshooting)

---

## 1. Repo Layout

Zealot is a monorepo with two parallel workspaces: a Rust workspace (`Cargo.toml`) and an npm workspace (`package.json`).

```
zealot/
├── Cargo.toml              Rust workspace: crates/* + apps/server,cli,tui,mcp
├── package.json            npm workspace: packages/* + apps/web,mobile
├── docker-compose.yml      Full-stack local stack (server, web, mcp)
├── rust-toolchain.toml     Pins Rust stable + rustfmt, clippy
├── tsconfig.base.json      Base TypeScript config extended by all packages
│
├── crates/                 Rust library crates (shared by all Rust apps)
│   ├── zealot-domain/      Data structures: Item, Attribute, Rule, Comment, …
│   ├── zealot-app/         Services (business logic), repo traits, port interfaces
│   ├── zealot-infra/       Concrete DB repos + port implementations (bcrypt, auth, …)
│   ├── zealot-api/         Axum HTTP handlers, middleware, router
│   └── zealot-lua/         Sandboxed Lua 5.4 rules engine (mlua)
│
├── apps/                   Runnable binaries
│   ├── server/             Rust REST API daemon (port 8456)
│   ├── web/                Vite SPA → nginx (port 8085 in Docker, 5173 in dev)
│   ├── mobile/             Tauri 2.0 cross-platform app (iOS + Android)
│   ├── cli/                Rust CLI — early stage
│   ├── tui/                Rust TUI — early stage
│   └── mcp/                Rust MCP server for LLM agents (port 3100)
│
├── packages/               TypeScript library packages
│   ├── domain/             Domain types mirroring the Rust domain structs
│   ├── api/                Typed HTTP clients, one file per API resource
│   ├── engine/             Shared utilities: HTTP helpers, events, hotkeys, nav
│   ├── ui/                 Web Components: screens, views, tools, shell
│   ├── content/            SVG icons (as TS exports) + global CSS
│   └── core/               Minimal shared utilities
│
├── database/               SQL schemas by domain area (item/, account/, repeat/, …)
│   │                       Both .sql (generic) and .psql (PostgreSQL) variants
│   └── init.sql            Database initialisation entry point
│
├── docs/                   Markdown documentation
│   └── ai-tasks/           Task tickets for the frontend rewrite (TASK-001 …)
│
├── scripts/                Build, deploy, and code-gen helpers
│   ├── .zealot.example.env Example server environment variables
│   └── .db.example.env     Example PostgreSQL credentials
│
└── test/
    ├── e2e/                Python pytest smoke tests (own Docker stack)
    └── playwright/         Playwright browser tests (own Docker stack)
```

### Crate dependency order

Dependencies flow strictly in one direction:

```
zealot-domain  ←  zealot-app  ←  zealot-infra  ←  zealot-api  ←  server
                                  zealot-lua ──────────────────────────┘
```

`zealot-domain` has no internal dependencies. `zealot-app` depends only on domain. Nothing in `domain` or `app` touches a database or network directly — that is `infra`'s job.

---

## 2. Local Development

### 2.1 Prerequisites

| Tool | Min version | Required for |
|---|---|---|
| Git | any | cloning the repo |
| Docker + Compose | Docker 24+ | any Docker-based workflow |
| Rust (via `rustup`) | 1.89 stable | native backend development |
| Node.js | 22 | web frontend development |
| Python 3 | 3.9+ | E2E tests only |

`rust-toolchain.toml` pins the exact Rust version. `rustup` reads this file and installs the right toolchain automatically on first `cargo` invocation.

### 2.2 Option A — Full stack with Docker (recommended for first run)

```bash
git clone <repo-url>
cd zealot
npm run docker
# Open http://localhost:8085
# First login creates your account.
```

This builds the `zealot-server`, `zealot-web`, and `zealot-mcp` images and starts all three containers. Data persists in a Docker volume named `zealot_data`.

### 2.3 Option B — Native development

Run the backend and frontend in separate terminals for faster iteration cycles with hot reload.

**Terminal 1 — backend:**

```bash
cargo run -p zealot-server
# Listens on http://localhost:8456
```

**Terminal 2 — frontend:**

```bash
npm install          # first time only
npm run dev:web
# Vite dev server on http://localhost:5173
# /api/* is proxied to http://localhost:8456 automatically
```

### 2.4 Environment variables

When running the server outside Docker, copy `scripts/.zealot.example.env` and set variables in your shell or a `.env` file.

**Server:**

| Variable | Default (Docker) | Description |
|---|---|---|
| `DATABASE` | `sqlite` | `sqlite`, `postgres`, or `mysql` |
| `DB_FILENAME` | `/data/zealot.db` | SQLite file path |
| `PORT` | `8456` | HTTP listen port |
| `ACCOUNT_CREATION_ENABLED` | `true` | Disable after first account to prevent open registration |
| `MEDIA_SOURCE` | `FILESYSTEM` | `FILESYSTEM` or `S3` |
| `MEDIA_PATH` | `/data/public` | Media storage root directory |
| `DB_DATABASE`, `DB_USERNAME`, `DB_PASSWORD`, `DB_HOSTNAME`, `DB_PORT` | — | PostgreSQL / MySQL only |
| `SESSION_STORE` | in-memory | `redis` for persistent sessions across restarts |
| `REDIS_HOST`, `REDIS_PORT`, `REDIS_DATABASE`, `REDIS_PASSWORD`, `REDIS_RESET` | — | Redis only |

**MCP server** (see `scripts/mcp-compose.sample.yml`):

| Variable | Default | Description |
|---|---|---|
| `ZEALOT_URL` | `http://server:8456` | Backend base URL |
| `ZEALOT_API_KEY` | _(empty)_ | API key — leave empty to use session auth |
| `MCP_MODE` | `http` | `http` or `stdio` |
| `MCP_PORT` | `3100` | MCP HTTP listen port |

### 2.5 Command reference

| Command | What it does |
|---|---|
| `npm run docker` | Build images + start full stack (server + web + mcp) |
| `npm run build:docker` | Build Docker images without starting containers |
| `npm run dev:web` | Vite dev server on port 5173 with HMR |
| `npm run build:web` | Production build → `apps/web/dist/` |
| `npm run typecheck` | `tsc --noEmit` across all packages |
| `npm run lint` | ESLint across all packages |
| `cargo run -p zealot-server` | Run backend in dev mode |
| `cargo build --release -p zealot-server` | Release-optimised backend build |
| `cargo test` | Run all Rust unit tests |
| `cargo test -p zealot-lua` | Run tests for a single crate |
| `cargo clippy` | Lint all Rust code |
| `./test/e2e/run.sh` | Run Python E2E smoke tests |
| `./test/playwright/run.sh` | Run Playwright browser tests |

---

## 3. Architecture

### 3.1 Data flow

```
Browser / Mobile / MCP client
          │
          │  HTTP JSON  (port 8456)
          ▼
    apps/server
          │
          ▼
  crates/zealot-api          Axum router + handlers
          │                  One file per resource: item.rs, comment.rs, rule.rs, …
          ▼
  crates/zealot-app          Service layer + repo interfaces
    ├── ItemService           Business logic; validates, orchestrates
    ├── PlannerService
    ├── RuleService ─────────► crates/zealot-lua   (Lua 5.4 sandbox)
    └── Ports                 Capability interfaces (email, auth, notifications)
          │
          ▼
  crates/zealot-infra        Concrete implementations
    ├── DB repos              sqlx queries against SQLite / PostgreSQL / MySQL
    └── Port impls            bcrypt, session, …
          │
          ▼
      Database
```

### 3.2 Rust crates

| Crate | Responsibility |
|---|---|
| `zealot-domain` | Pure data: structs, enums, DTO types, error types. Zero I/O. |
| `zealot-app` | Business logic and orchestration. Defines repo traits and port interfaces — no concrete implementations here. |
| `zealot-infra` | Implements the repo traits with `sqlx`. Implements port interfaces (bcrypt hashing, session tokens, etc.). |
| `zealot-api` | Axum router, handler functions, middleware (CORS, auth, CSRF), request/response deserialization. |
| `zealot-lua` | `mlua`-backed Lua 5.4 sandbox. Exposes a `zealot.*` API object to user scripts. Isolated from the database — communicates back through the service layer. |

**Source of truth for the HTTP contract:** `crates/zealot-api/src/http/`. When verifying what a route accepts or returns, read the handler — not the TypeScript client.

### 3.3 Domain model

All entities are defined in `crates/zealot-domain/src/`:

| Entity | File | What it represents |
|---|---|---|
| `Item` | `item.rs` | The universal unit: wiki page, task, note, goal, meeting. |
| `ItemType` | `item_type.rs` | User-defined templates that give items a category (Task, Project, Meeting). Items can have multiple types. |
| `AttributeKind` / `Attribute` | `attribute.rs` | Typed key-value fields on items. Kinds are defined once per account (Status, Date, Priority). Attributes are per-item values. |
| `ItemLink` | `item.rs` | Named relationships between items: `parent`, `blocks`, `tag`, `topic`, `other`. |
| `Comment` | `comment.rs` | Timestamped annotation attached to an item. |
| `Repeat` / `RepeatEntry` | `repeat.rs` | Recurring task configuration + per-day completion status tracking. |
| `Rule` | `rule.rs` | Lua automation script with a trigger (event, cron schedule, or manual). |
| `Account` / `Session` | `account.rs`, `auth.rs` | Users and authentication tokens. |

### 3.4 HTTP API routes

Handlers live in `crates/zealot-api/src/http/`. The routes are mounted at `/` in the server binary:

| Prefix | Handler file | Covers |
|---|---|---|
| `/item` | `item.rs` | Create, list, search, export (PDF/DOCX) items |
| `/item/{id}` | `item.rs` | Get, update, delete a single item |
| `/item/{id}/attr` | `attribute.rs` | Read/write item attribute values |
| `/item/{id}/assign_type/{name}` | `item.rs` | Add / remove a type from an item |
| `/item_type` | `item_type.rs` | Manage type definitions |
| `/attribute` | `attribute.rs` | Manage attribute kind definitions |
| `/comment` | `comment.rs` | Create, read, update, delete comments |
| `/planner` | `planner.rs` | Daily, weekly, monthly planner views |
| `/repeat` | `repeat.rs` | Recurring task scheduling |
| `/rule` | `rule.rs` | Create, list, run automation rules |
| `/media` | `media.rs` | File upload and retrieval |
| `/auth` | `auth.rs` | Login, register, API key management |
| `/account` | `account.rs` | User profile |
| `/health` | `health.rs` | Liveness check |

### 3.5 Persistence

SQL schemas live in `database/` by domain area (`account/`, `item/`, `repeat/`, etc.). Each area has both a `.sql` (generic) and `.psql` (PostgreSQL-specific) variant.

SQLite is the default — no external service needed for local development. To use PostgreSQL, set `DATABASE=postgres` and provide the `DB_*` variables.

Migration approach is documented in [migrations.md](./migrations.md).

### 3.6 TypeScript packages

The frontend is split into focused packages under `packages/`:

```
packages/domain/    TypeScript types mirroring the Rust domain structs
packages/api/       Typed fetch wrappers — one file per API resource
                    (item.ts, comment.ts, rule.ts, auth.ts, …)
packages/engine/    Shared utilities:
                      api/    — HTTP helpers (get_json, post_json, withCsrf)
                      ui/     — base element classes, hotkeys, popups, nav
                      logic/  — event emitter (mitt) for cross-component events
packages/ui/        Web Components — screens, views, tools, shell
packages/content/   SVG icon modules + global CSS assets
```

**Key constraints:**
- Never make `fetch` calls from `packages/ui/`. All HTTP goes through `packages/api/`.
- Never embed raw SVG in component files. Import icon modules from `packages/content/src/`.
- Never add a framework (React, Vue, etc.). All UI is vanilla Web Components.

### 3.7 Web Components pattern

Every UI element is an `HTMLElement` subclass registered via `customElements.define()`. Component-local styles go in a `<style>` tag inside the shadow DOM. For the canonical pattern, read any existing component under `packages/ui/src/`.

### 3.8 MCP server

`apps/mcp/` exposes Zealot as a Model Context Protocol server for LLM agents. Tools are defined in `apps/mcp/src/tools/`:

| Tool file | Capabilities |
|---|---|
| `wiki.rs` | CRUD on items: read, create, update, search |
| `planner.rs` | View and manage planner entries + repeats |
| `automation.rs` | List and execute Lua rules |
| `media.rs` | Upload and retrieve media attachments |

Transport modes: `stdio` (for local Claude Code integration) or `http` (for remote AI services). See [mcp-system-prompt.md](./mcp-system-prompt.md) for integration details.

---

## 4. Testing

### 4.1 Test suites

| Suite | Runner | Command | What it covers |
|---|---|---|---|
| Rust unit tests | `cargo test` | `cargo test` | Domain structs, service logic, Lua engine |
| Single-crate tests | `cargo test` | `cargo test -p <crate>` | Isolated crate, e.g. `cargo test -p zealot-lua` |
| E2E smoke tests | Python pytest | `./test/e2e/run.sh` | API routes, auth, DB, rules engine via HTTP |
| Browser tests | Playwright | `./test/playwright/run.sh` | Full UI flows in a headless browser |

### 4.2 E2E smoke tests (`test/e2e/`)

`run.sh` manages the full lifecycle:

1. Creates a Python virtualenv and installs `pytest` + `requests`
2. Starts `test/e2e/compose.yml` — server on port **18456**, web on port **18080**
3. Polls until health checks pass
4. Runs pytest
5. Tears down the stack (with `-v` to remove volumes)

Pass extra pytest args directly: `./test/e2e/run.sh -k auth -q`

Test files:

| File | Covers |
|---|---|
| `test_smoke.py` | Health endpoint, basic reachability |
| `test_auth.py` | Registration, login, session, API keys |
| `test_database.py` | Item CRUD, attribute operations |
| `test_rules_engine.py` | Rule creation, manual execution, output log |

### 4.3 Playwright browser tests (`test/playwright/`)

`run.sh` manages the full lifecycle:

1. Tears down any leftover containers
2. Builds and starts `test/playwright/compose.yml` — web on port **18081**
3. Installs Playwright browsers if missing
4. Runs the test suite

From `test/playwright/` you can also run individual modes:

```bash
npm run test          # headless (CI default)
npm run test:headed   # with browser window visible
npm run test:ui       # interactive Playwright UI
npm run test:debug    # debug mode with pause
npm run report        # view last HTML report
```

### 4.4 TypeScript type checking

```bash
npm run typecheck    # tsc --noEmit across all packages
```

Run this before every commit that touches TypeScript. It catches type errors across package boundaries that a single-package build would miss.

---

## 5. First Contribution

This walkthrough traces a concrete end-to-end change: **adding an `author_name` field to the `CommentDto`** so comments show who wrote them.

### Step 1 — Read the relevant code

Start with the domain struct and the HTTP handler for comments:

- `crates/zealot-domain/src/comment.rs` — `CommentDto`, `AddCommentDto`
- `crates/zealot-api/src/http/comment.rs` — route handlers

For rewrite tasks from `docs/ai-tasks/rewrite/`, also read the original client code:

```bash
git diff master -- client/src/features/<feature>/
```

### Step 2 — Add the field to the domain struct

In `crates/zealot-domain/src/comment.rs`, add `author_name` to `CommentDto`:

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CommentDto {
    pub comment_id: i64,
    pub item: ItemDto,
    pub timestamp: String,
    pub content: String,
    pub author_name: String,   // new field
}
```

### Step 3 — Propagate through the service layer

In `crates/zealot-app/src/`, update the service that builds `CommentDto` to populate `author_name` from the account associated with the comment. Update the repo trait in `zealot-app/src/repos/` if a new query is needed.

### Step 4 — Update the infrastructure

In `crates/zealot-infra/src/`, update the `sqlx` query to JOIN the `account` table and return the author's name. Map it into the new field.

### Step 5 — Verify the API compiles

```bash
cargo build -p zealot-server
```

Fix any type errors before touching TypeScript.

### Step 6 — Update the TypeScript domain type

In `packages/domain/src/`, add `author_name: string` to the `CommentDto` interface.

### Step 7 — Update the API client

In `packages/api/src/comment.ts`, the existing fetch wrapper returns `CommentDto`. No change needed unless a new endpoint was added — the new field flows through automatically.

### Step 8 — Update the UI component

In `packages/ui/src/`, find the component that renders comments. Add a line to display `comment.author_name`.

### Step 9 — Verify

```bash
npm run typecheck               # TypeScript errors across all packages
cargo test                      # Rust unit tests
./test/e2e/run.sh               # E2E smoke tests
```

### General principles

- The backend handler in `crates/zealot-api/src/http/` is the contract. Match it exactly in the TypeScript client.
- Changes that cross the Rust/TypeScript boundary require updates in both — check both after any API shape change.
- `npm run typecheck` catches cross-package type errors that per-package builds miss.
- Mark intentional deviations from original behaviour with a short code comment explaining why.

---

## 6. Troubleshooting

| Symptom | Likely cause | Fix |
|---|---|---|
| `npm run docker` fails immediately | Docker daemon not running | Start Docker Desktop or `sudo systemctl start docker` |
| Docker build fails during Rust compilation | Network timeout fetching crates | Retry — the crates.io registry can be flaky |
| `cargo run -p zealot-server` errors: wrong Rust version | Rustup not updated | `rustup update stable` |
| `cargo run -p zealot-server` errors: cannot open DB file | SQLite path not writable | Set `DB_FILENAME` to a writable path, e.g. `/tmp/zealot.db` |
| Web at `:5173` returns 502 on any `/api/*` request | Backend not running | Start `cargo run -p zealot-server` first |
| `npm run dev:web` fails with TypeScript errors | Stale or missing deps | Run `npm install` from the repo root |
| `npm run typecheck` fails with "cannot find module" | Package build artifact missing | Run `npm run build:web` once, then retry |
| E2E tests time out waiting for services | Docker image build is slow | Let it finish — first build takes a few minutes; subsequent runs are faster |
| E2E tests fail: "connection refused" | Port conflict | Check that ports 18456 and 18080 are free: `lsof -i :18456` |
| Playwright: "browser not found" | First run, browsers not installed | `run.sh` installs them automatically — just retry |
| MCP server: connection refused | Wrong `ZEALOT_URL` | Set `ZEALOT_URL=http://localhost:8456` when running outside Docker |
| Account creation returns 404 | `ACCOUNT_CREATION_ENABLED=false` | Set `ACCOUNT_CREATION_ENABLED=true` in your `.env` or shell |
| Mobile build fails (iOS) | Xcode not signed in | Open Xcode → Settings → Accounts → add your Apple ID |
| Mobile build fails (Android) | NDK not installed | Install NDK via Android Studio SDK Manager |
| `cargo clippy` warns about many things | Workspace-wide first run | Warnings are advisory; fix before submitting a PR |

---

## Related docs

- [Building & Running](./building.md) — full build instructions including mobile and Docker
- [Migrations](./migrations.md) — database schema migration system
- [Rules Engine](./rules-engine.md) — Lua automation reference and cookbook
- [Analysis Tooling](./analysis-tooling.md) — Rust coupling and TypeScript dependency analysis
- [MCP System Prompt](./mcp-system-prompt.md) — Zealot MCP tools for LLM agents

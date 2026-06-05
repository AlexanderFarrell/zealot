# Zealot Quickstart

This guide gets you from a fresh machine to using Zealot for real planning and note-taking in one session. It covers installation, first login, and a worked example you can follow step by step.

---

## What is Zealot?

Zealot is a personal wiki and planner that runs entirely on your own machine or server. Every piece of information — a note, a task, a goal, a meeting — is an **item**. Items can have structured fields (attributes), belong to categories (types), link to each other, and appear on a date-based planner. There is no vendor cloud; your data stays where you put it.

---

## Prerequisites

You need **Docker** and **Docker Compose**. That is all.

- [Install Docker Desktop](https://docs.docker.com/get-docker/) — includes Compose on macOS and Windows
- On Linux, install the Docker Engine package and the `docker-compose-plugin` separately

Verify both are working:

```
docker --version
docker compose version
```

Both commands should print a version number without errors.

---

## Installation

### 1. Download the source

```
git clone https://github.com/AlexanderFarrell/zealot.git
cd zealot
```

> **Note:** Pre-built images and a standalone download are not yet published. You are building from source with Docker.

### 2. Start Zealot

```
npm run docker
```

This builds the backend and web frontend images and starts all services. The first build takes a few minutes. When you see the containers settle (no new log lines), Zealot is ready.

Open your browser at **http://localhost:8085**.

### Where data lives

All your data (the SQLite database and any uploaded files) is stored in a Docker volume named `zealot_data`. It persists across `docker compose down` and restarts. To inspect it:

```
docker volume inspect zealot_zealot_data
```

To back it up, copy the SQLite file from the volume's mount path to a safe location.

---

## First login

Zealot uses accounts. The first time you open the app you will see a register form — fill in a username and password and submit. This creates your account. Future visits use the login form with the same credentials.

Each account's data is isolated. If you are the only person running this instance, one account is enough.

---

## Your first item

Zealot's entire model rests on one concept: the **item**. An item is just a record with a title and a body. You can think of it as a wiki page, a task, a note, or anything else — the label you give it is up to you.

Click **New item** (or the equivalent button in the sidebar), give it a title like `Intro Note`, and write something in the body. The body uses ZealotScript, which is GitHub-flavoured Markdown with a few extras. Plain text works fine to start.

Click save. You now have your first item.

---

## Core concepts

Before going further, here are the four building blocks you will use constantly.

### Items

An item is a titled document with a freeform body. Every other concept in Zealot attaches to items.

### Types

Types are categories you define. Examples: *Task*, *Project*, *Goal*, *Meeting*. An item can have zero types or many. Types do not change what an item is — they are just labels that let you filter and organise. You define your own types; Zealot does not impose a built-in hierarchy.

### Attributes

Attributes are structured fields — typed key-value pairs you add to an item. Examples: `Status = "In Progress"`, `Due Date = 2026-06-30`, `Priority = "High"`. You define which attribute names and types exist (text, date, integer, dropdown, etc.), then set them on individual items.

Attributes power the planner: any item with a `Date` attribute (in `YYYY-MM-DD` format) appears on the day planner for that date.

### Links

Items can be linked to each other. The most common relationship is **parent** — it makes one item a child of another, creating a hierarchy. Other relationships include *blocks*, *tag*, *topic*, and *other*. You can also create **wiki links** inside body text: `[[Other Item Title]]` becomes a clickable link.

---

## Worked example: a simple project system

The following walkthrough builds a small real-world system from scratch. It takes about ten minutes.

### Step 1 — Create item types

Go to **Item Types** in the sidebar (or settings area).

Create three types:
- `Project`
- `Task`
- `Daily Plan`

You do not need to configure anything else at this stage.

### Step 2 — Create attribute kinds

Go to **Attributes** (or wherever attribute kinds are managed).

Create these attribute kinds:

| Key | Type | Notes |
|---|---|---|
| `Status` | dropdown | Values: `Not Started`, `In Progress`, `Complete`, `Blocked` |
| `Date` | date | Used by the planner to schedule items |
| `Priority` | dropdown | Values: `Low`, `Medium`, `High` |

These are shared definitions. Once created, any item can use them.

### Step 3 — Create a project

Create a new item:

- **Title:** `Website Redesign`
- **Type:** assign `Project`
- **Body:** A brief description of what this project is

### Step 4 — Create tasks under the project

Create three items, each with **Type** set to `Task`:

- `Write new homepage copy`
- `Design mockups`
- `Set up staging environment`

For each task:
1. Open the item
2. Set the **parent** link to `Website Redesign`
3. Set the `Status` attribute to `Not Started`
4. Optionally set `Priority`

Now if you open `Website Redesign` and view its children, you will see all three tasks listed under it.

### Step 5 — Schedule work on the planner

Suppose you want to work on `Write new homepage copy` today.

Open that item and set its `Date` attribute to today's date (e.g. `2026-06-05`).

Navigate to the **Planner** and select today's date. The item appears in your day view.

This is the core planning loop: set a `Date` on any item to block time for it. Move the date to reschedule.

### Step 6 — Create a daily plan note

Create one more item:

- **Title:** `Daily Plan — 2026-06-05`
- **Type:** `Daily Plan`
- **Date attribute:** `2026-06-05`
- **Body:**

```
## Focus
Write new homepage copy

## Also
- Review design brief
- Check email

## Notes
Working from home today. Block notifications after 10am.
```

This gives you a freeform scratchpad for the day alongside your scheduled task items. Both appear on the planner for the same date.

### What you have built

```
Website Redesign  [Project]
├── Write new homepage copy  [Task]  Status: Not Started  Date: 2026-06-05
├── Design mockups           [Task]  Status: Not Started
└── Set up staging server    [Task]  Status: Not Started

Daily Plan — 2026-06-05  [Daily Plan]  Date: 2026-06-05
```

This is a minimal but complete planning system. Projects contain tasks, tasks appear on the day planner when you give them a date, and a daily plan note keeps your day in focus.

---

## Writing content with ZealotScript

Item bodies support **ZealotScript** — Markdown with a few extensions.

```
# Heading 1
## Heading 2

**bold**   *italic*   ~~strikethrough~~   _underline_

- Bullet list item
1. Numbered list item

[[Other Item Title]]        ← wiki link to another item
[[type:Project]]            ← link to a type page

:::note
This is a callout block. Also: warning, tip, danger, info, success.
:::

$E = mc^2$                  ← inline math
```

Wiki links (`[[...]]`) auto-complete as you type and are rendered as clickable links that open the referenced item. This is the primary way to cross-reference items.

---

## Searching

Use the search bar (usually at the top of the sidebar) to find items by title keyword. Results appear as you type.

To filter by type, use the type filter in the item list view — for example, show only items of type `Task`.

The planner shows items by date. Recent items are also available from the sidebar.

---

## API keys (optional)

If you want to access Zealot from the mobile app or an external tool like the MCP integration, you need an API key instead of a session cookie.

In your account settings, generate a new API key and give it a label (e.g. `Mobile`). Copy the key immediately — it is shown only once. Paste it into the mobile app or tool that needs it.

API keys have the same access level as your account. Revoke them from account settings if you no longer need them.

---

## What to read next

- [ZealotScript reference](./zealotscript/README.md) — full markup syntax
- [Rules Engine](./rules-engine.md) — automate Zealot with Lua scripts (scheduled reports, status stamps, etc.)
- [MCP integration](./mcp-system-prompt.md) — use Zealot with LLM agents
- [Building from source](./building.md) — run the backend and frontend without Docker for development

---

## Assumptions and known gaps

The following steps in this guide were written from reading the source code rather than a live fresh install. They are believed to be accurate but have not been verified against the current Docker image:

- The exact location of the **Item Types** and **Attributes** management UI may differ from the description above. The underlying API endpoints (`/item_type` and `/attribute`) are confirmed in the backend source.
- Pre-built Docker images are not yet published. `npm run docker` builds locally.
- The register screen is served at the root URL when no session is active; this is the expected first-run flow based on the auth handler code.

If something does not match what you see, please open an issue.

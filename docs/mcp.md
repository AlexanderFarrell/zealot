# Zealot MCP Guide

Zealot exposes a full [Model Context Protocol](https://modelcontextprotocol.io/) server so that AI agents — Claude Desktop, OpenAI Codex CLI, LibreChat, or any other MCP-compatible client — can read and write your wiki and planner directly.

This guide is for **humans setting up and operating** the MCP integration. For the agent-facing system prompt that describes Zealot's data model, see [MCP System Prompt](./mcp-system-prompt.md).

---

## Overview

The MCP server bridges your AI client and the Zealot REST API. Every tool call the agent makes is translated into an authenticated HTTP request against Zealot, so the agent can:

- Browse, create, and update wiki items and their attributes
- Read your daily, weekly, and monthly planner
- Log and review comments on items
- Check and update habit/repeat status
- Create and run Lua automation rules
- Manage files in the media library

The server supports two transport modes:

| Mode | When to use |
|---|---|
| **stdio** | Claude Desktop running locally on the same machine as Zealot |
| **http** | Remote or web-based clients (LibreChat, Codex CLI with a remote Zealot instance, etc.) |

---

## Why use the MCP?

The MCP integration lets you talk to your Zealot data in plain English instead of constructing API calls. An AI agent — Claude Desktop, Codex CLI, LibreChat — reads and writes your wiki and planner on your behalf, turning natural-language requests into the right sequence of tool calls.

- **Query your second brain.** Ask "what did I work on this week?", "what are my open goals?", or "am I falling behind on my exercise habit?" and the agent reads your Zealot data and answers — no UI, no searching.
- **Capture and organise on the fly.** "Add a task called 'follow up with legal' under the Q3 Roadmap, due Friday, priority High" — the agent finds the project, creates the item, sets the attributes, and confirms what it did.
- **Automation co-pilot.** Describe a rule in English ("stamp a Finished Date whenever Status is set to Complete") and the agent writes the Lua script, shows it to you for review, and installs it.
- **Daily briefing.** Start your morning by asking for a summary of today's scheduled items, pending habits, and overdue tasks — without opening a browser.

Example prompt to try with Claude Desktop once connected:

```
Show me today's plan, mark my morning standup habit as complete,
and create a task called "Review Q3 metrics" due this Friday.
```

---

## Installation and configuration

### Prerequisites

- A running Zealot instance (see [Quickstart](./quickstart.md))
- A Zealot API key — generate one in the Zealot web UI under **Settings → API Keys**, or via the HTTP API (`POST /auth/api-key`)

### Environment variables

| Variable | Default | Required | Purpose |
|---|---|---|---|
| `ZEALOT_API_KEY` | — | **Yes** | API key sent as the `X-API-Key` header on every request |
| `ZEALOT_URL` | `http://localhost:7377` | No | Base URL of your Zealot API server |
| `MCP_MODE` | `stdio` | No | Transport mode: `stdio` or `http` |
| `MCP_PORT` | `3100` | No | Listening port when `MCP_MODE=http` |

All variables can also be passed as CLI flags (`--api-key`, `--url`, `--mode`, `--port`).

### Stdio mode — Claude Desktop

Stdio mode connects the MCP server to Claude Desktop via stdin/stdout. The server binary must be available on your `PATH` or referenced by absolute path.

Add the following to your Claude Desktop configuration file (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS, `%APPDATA%\Claude\claude_desktop_config.json` on Windows):

```json
{
  "mcpServers": {
    "zealot": {
      "command": "zealot-mcp",
      "env": {
        "ZEALOT_URL": "http://localhost:7377",
        "ZEALOT_API_KEY": "your-api-key-here"
      }
    }
  }
}
```

Restart Claude Desktop after saving. You should see "zealot" appear in the MCP servers list.

If you built the server from source, replace `"zealot-mcp"` with the absolute path to the binary, e.g. `"/home/you/projects/zealot/target/release/zealot-mcp"`.

### HTTP mode — Docker Compose

HTTP mode starts a web server that any MCP-over-HTTP client can connect to. The endpoint is `http://<host>:<port>/mcp`.

```yaml
services:
  zealot_mcp:
    image: registry.alexanderfarrell.net/zealot-mcp:latest
    container_name: zealot-mcp
    restart: unless-stopped
    ports:
      - "3100:3100"
    environment:
      ZEALOT_URL: "https://zealot.example.com"
      ZEALOT_API_KEY: "your-api-key-here"
      MCP_MODE: "http"
      MCP_PORT: "3100"
    healthcheck:
      test: ["CMD", "wget", "--spider", "-q", "http://localhost:3100/health"]
      interval: 10s
      timeout: 3s
      retries: 5
```

A sample file is available at `scripts/mcp-compose.sample.yml`.

### HTTP mode — other clients

Point your MCP client at `http://<host>:<port>/mcp`. The health endpoint at `/health` returns `ok` when the server is running.

**Codex CLI** (with a remote server):

```bash
codex --mcp-server "http://zealot-mcp.example.com:3100/mcp" "Show me today's plan"
```

**LibreChat** and similar web clients that don't support Server-Sent Events are handled automatically — the server's SSE-to-JSON compatibility layer converts streaming responses to plain JSON.

---

## Tool Reference

The authoritative agent-facing tool contract lives in [MCP System Prompt](./mcp-system-prompt.md). Keep detailed tool names, parameter shapes, and workflow idioms there so MCP clients do not receive conflicting instructions from two documents.

At a high level, the server covers these workflows:

- Wiki items: browse, search, filter, line-level grep, outline, section and line-range reads, create, update, exact-text edits, section rewrites, append, delete, type assignment, attributes, and links.
- Planner and habits: day dashboards, period plans, habit entries, habit status updates, comments, and journal entries.
- Time blocks: range/item reads plus create, update, and delete operations using `HH:MM` times.
- Media: list, read, upload, create folder, rename, and delete.
- Analysis: wiki stats, orphan samples, attribute usage, and habit statistics.
- Automation: list, inspect, create, update, delete, and run Lua rules.
- Statistics: list series, record, edit, and delete entries, and read raw entries, daily series, and summaries.
- Saved Views: run, inspect, and validate `Saved View` items, whose content holds a `zealot-view` query definition.
- Schema: item types and attribute kinds.

`apps/mcp/tests/docs_tests.rs` fails when either MCP doc names a tool the server does not register, or when the system prompt omits a registered tool. When checking a live server, prefer your MCP client's tool list over this human guide; a live server missing tools listed here is running an older build. If the client exposes an older tool surface, use the closest legacy tool names but keep the same operating rules: browse before deep reads, page until complete, verify schema before unfamiliar writes, and read before overwriting content.

---

## Example Prompts And Workflows

These examples describe intent and expected tool strategy rather than exact call syntax. See [MCP System Prompt](./mcp-system-prompt.md) for current call shapes.

### 1. Creating a Ticket

**Prompt:** "Create a task called 'Fix login bug' under the Auth project, due this Friday, priority High"

Expected strategy:

- Search or filter for the Auth project and preserve its item id.
- Check live type and attribute schema if the task/ticket conventions are unfamiliar.
- Create the item with content, attributes, type, and parent/link in one call when supported.
- If using an older tool surface, create the item first, then set attributes, assign the type, and establish the parent/link using the available legacy tools.
- Confirm destructive or ambiguous changes before writing.

### 2. Planning A Day

**Prompt:** "Show me today's plan and mark my morning standup habit as complete"

Expected strategy:

- Prefer the composite day dashboard tool when available; otherwise gather the day plan, habit entries, time blocks, and comments separately.
- Identify the habit by id from the live result.
- Update habit status with one of the supported status values.

### 3. Reading A Project

**Prompt:** "Give me a summary of the Q3 Roadmap project and its open tasks"

Expected strategy:

- Resolve the project by exact title or search result, then use its id.
- Use an outline or summary view before reading long content.
- Read children or linked items through the current link tool; fall back to older child-list tools if needed.
- Fetch full item details only for the project and relevant open tasks.

### 4. Updating A Habit With A Note

**Prompt:** "I exercised today but only for 20 minutes; log it as alternate with a note"

Expected strategy:

- Read habit entries for the date or dashboard.
- Resolve the exercise habit id.
- Set the habit status to `Alternate` with the user's note.

---

### 5. Recording And Reviewing Statistics

Use `list_statistics` to discover series, `record_statistic_entry` to add a value, and
`get_statistic_entries` for raw history. `get_statistic_daily_series` applies the configured UTC
daily rollup. `get_statistic_summary` returns count, first, latest, minimum, maximum, average, sum,
and latest-minus-first delta. `edit_statistic_entry` and `delete_statistic_entry` use the stable
Statistic Entry ID.

---

## Safe Operating Rules For Agents

These rules help agents avoid data loss or unintended changes. Paste them into your system prompt or reference this guide when configuring agent behavior.

### Read before writing

Always resolve and inspect the target item before calling update or delete tools. Confirm the item is correct before changing it, and read existing content before overwriting content fields.

### Destructive operations require user confirmation

Before executing any of the following, the agent should present what it is about to do and ask the user to confirm:

- `delete_item` — permanently removes the item and all its data
- `delete_comment` — cannot be undone
- `delete_rule` — removes the automation rule
- `delete_media` — permanently removes the file or empty folder
- `delete_item_type` — removes the type definition (items keep their data, but lose the type label)
- `update_attribute_kind` or `delete_attribute_kind` changes affect every item using that attribute

### Attribute Updates Are Merge, Not Replace

Attribute update tools only overwrite the keys you pass unless the specific tool documents full replacement semantics. Existing attributes not included in the call are normally left unchanged, so you do not need to read and re-send all attributes to update one field.

### Item Content Updates Are Targeted

Change part of an item with `edit_item` (exact text) or `replace_section` (one heading's section) instead of resending the body. Find the spot with `grep_items` or `get_item_outline`, read it with `read_item`, and pass the `version` from that read as `expected_version`: if the item changed in the meantime, the write is refused with a conflict instead of silently overwriting the other edit. `append_to_item` appends on the server.

`update_item` updates only the fields you pass. Omitting `content` leaves the existing content unchanged. Omitting `title` leaves the title unchanged. Passing `content` replaces the whole body. `update_item` changes only title and content: to change attributes, including `Parent` and `Status`, use `update_item_attributes` with `set`, `rename`, and `remove`.

### Unknown Parameters Are Rejected

Every tool rejects a parameter it does not define. The error names the field and lists the accepted ones, for example ``unknown field `attributes`, expected one of `item`, `set`, `remove`, `rename` ``, and nothing is written. `update_item_attributes` also rejects a call whose `set`, `rename`, and `remove` are all absent or empty, and its reply lists the keys it set, renamed, and removed, so a call that changed nothing cannot look like one that did. The server logs an `item attributes changed` line for each write.

### Show automation scripts before running

Before calling `create_rule`, `update_rule`, or `run_rule`, show the Lua script to the user and explain what it does. Automation rules run server-side with access to your full Zealot data.

### Schema changes are global

`create_item_type`, `update_item_type`, `delete_item_type`, `create_attribute_kind`, and `update_attribute_kind` affect every item that uses that type or attribute. Treat these like schema migrations — confirm intent before proceeding.

---

## Known limitations

### No export via MCP

Item export to PDF or DOCX is not exposed through MCP tools. Use the HTTP API directly: `GET /item/{id}/export?format=pdf`. See [HTTP API](./api.md).

### No batch operations

Each tool call is one item's worth of work, with one exception: `set_status` changes Status on up to 50 items and comments on each in one call. There is no other bulk create/update/delete and no batch item read by id list; for large operations the agent must make multiple sequential calls. `filter_items` (with `types`, `under`, `sort`, and `fields`) covers most scans in one call.

### Event rules do not chain

Automation rules with event triggers (`on_item_create`, `on_type_assign`, etc.) do not re-trigger other event rules. This prevents infinite loops but means you cannot chain event rules. Use a single rule that does everything, or combine event and cron triggers.

### MCP prompts require client support

The server defines 10 structured prompts (`daily_briefing`, `plan_my_week`, `capture_idea`, etc.) that guide the agent through common workflows. These are only available in clients that support the MCP prompts capability. Claude Desktop supports them; Codex CLI and most HTTP clients currently do not. Tools work in all clients regardless.

### HTTP mode compatibility

The HTTP server includes a compatibility layer that converts Server-Sent Events responses to plain JSON for clients that don't handle streaming. This is transparent to the agent but means some clients receive responses slightly later than native SSE clients.


## Reversible item archiving

`set_archived` takes `{item: "#123", archived: true, subtree: false}`. Use
`archived: false` to unarchive. Read the target first and obtain authority for the
items affected. `subtree: true` toggles this item and every descendant through
Parent links, including descendants with other parents, in one atomic,
account-bounded action. Cycles are safe. It does not change parents, types,
Status, content, comments, or any other attribute. Unarchiving a subtree clears
its archive flags, including flags set before the original subtree action.
New children do not inherit an ancestor's archive state.

State uses the existing boolean attribute primitive: `Archived: true` hides an
item; missing/false means visible. The API reports `archived` on full item reads
and MCP summaries. The existing Status remains independent. No items are
archived by migration, and no history is deleted or stripped.

`browse_items` (all modes), `filter_items`, `search_items`, `grep_items`,
`get_linked_items`, `get_plan`, and `day_dashboard` accept
`include_archived: true`; the default excludes archived discovery results.
`list_item_types` accepts the same flag for its item counts. Counts and paging
apply to the selected visible set. Direct `get_item`, exact-title reads,
content reads, and wikilinks continue to resolve archived items.

In the UI, use **Archive Item** / **Unarchive Item** on the item screen, or
**Archive Subtree** / **Unarchive Subtree** for descendants. The confirmation
states the effect. Search (Ctrl+O) has **Include archived** for discovery and
restoration; archived results and item screens show an archive label.

HTTP: `POST /item/{id}/archive` with `{archived: true|false, subtree: true|false}`
returns `{item_ids: [...], archived: ...}`. Item discovery, planner, and
`/item_type/summary` GET endpoints accept `?include_archived=true`;
`POST /item/query`, `/item/filter`, and `/item/grep` accept the boolean in their
body. Omitting the opt-in excludes archived items. Backup and direct history
reads preserve archived items.

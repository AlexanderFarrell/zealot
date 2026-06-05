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

## Tool reference

The MCP server exposes 38 tools grouped below by workflow.

### Items — wiki management

These tools cover the core Zealot data model. Every piece of information in Zealot is an item.

| Tool | Parameters | What it does |
|---|---|---|
| `list_items` | `type_filter?` | List root-level items, optionally filtered by type name (e.g. `"Goal"`, `"Project"`) |
| `list_recent_items` | `limit?` (default 30), `offset?` | List recently modified items with pagination |
| `search_items` | `term` | Search items by title keyword; results sorted by relevance |
| `get_item` | `id` | Fetch a single item by numeric ID — returns full attributes, types, and links |
| `get_item_by_title` | `title` | Fetch a single item by exact title |
| `get_children` | `id` | List all child items of a parent |
| `get_related_items` | `id` | List all items linked to this item, regardless of relationship type |
| `create_item` | `title`, `content`, `attributes?` | Create a new item |
| `update_item` | `id`, `title?`, `content?` | Update title and/or content (partial — omitted fields unchanged) |
| `delete_item` | `id` | Permanently delete an item |
| `set_item_attributes` | `id`, `attributes` | Set one or more attributes; only specified keys are overwritten, others preserved |
| `delete_item_attribute` | `id`, `key` | Delete a single attribute by key |
| `assign_item_type` | `item_id`, `type_name` | Assign a type to an item (items can have multiple types) |
| `unassign_item_type` | `item_id`, `type_name` | Remove a type from an item (item itself is not deleted) |

### Planner

| Tool | Parameters | What it does |
|---|---|---|
| `get_day_plan` | `date` (`YYYY-MM-DD`) | Get all items scheduled for a specific day |
| `get_week_plan` | `week` (`YYYY-WNN`, e.g. `"2026-W23"`) | Get all items scheduled for a week |
| `get_month_plan` | `month` (1–12), `year` | Get all items scheduled for a month |

### Comments

| Tool | Parameters | What it does |
|---|---|---|
| `get_comments_for_item` | `item_id` | Get all comments attached to an item |
| `get_comments_for_day` | `date` (`YYYY-MM-DD`) | Get all comments logged for a day (journal view) |
| `add_comment` | `item_id`, `timestamp` (`YYYY-MM-DD HH:MM:SS`), `content` | Add a timestamped comment to an item |
| `update_comment` | `comment_id`, `content` | Update comment body text |
| `delete_comment` | `comment_id` | Permanently delete a comment |

### Repeats / habits

| Tool | Parameters | What it does |
|---|---|---|
| `get_repeat_entries` | `date` (`YYYY-MM-DD`) | Get habit/repeat entries with completion status for a day |
| `update_repeat_status` | `item_id`, `date`, `status?`, `comment?` | Update habit completion; status is one of `Complete`, `Skip`, `Alternate`, `NotComplete` |

### Media

| Tool | Parameters | What it does |
|---|---|---|
| `list_media` | `path?` (relative; omit for root) | List files and folders with metadata |
| `create_media_folder` | `folder` | Create a folder (e.g. `"images/2026"`) |
| `delete_media` | `path` | Delete a file or empty directory |

### Automation rules

| Tool | Parameters | What it does |
|---|---|---|
| `list_rules` | — | List all automation rules |
| `get_rule` | `rule_id` | Get a single rule including its full Lua script |
| `create_rule` | `name`, `script`, `trigger`, `description?`, `enabled?` | Create a new rule |
| `update_rule` | `rule_id`, `name?`, `script?`, `trigger?`, `description?`, `enabled?` | Update rule properties |
| `delete_rule` | `rule_id` | Delete a rule |
| `run_rule` | `rule_id` | Run a rule immediately regardless of its configured trigger |

Trigger formats:

```json
{"kind": "manual"}
{"kind": "cron", "expression": "0 9 * * *"}
{"kind": "on_item_create"}
{"kind": "on_type_assign", "type_name": "Goal"}
```

See [Rules Engine](./rules-engine.md) for the full Lua API reference.

### Schema — item types and attribute kinds

These tools manage the type system. Changes here affect all items that use a given type or attribute.

| Tool | Parameters | What it does |
|---|---|---|
| `list_item_types` | — | List all item types |
| `get_item_type_by_name` | `name` | Get type definition including required attributes |
| `create_item_type` | `name`, `description?`, `icon?`, `color?` | Create a new item type |
| `update_item_type` | `type_id`, `name?`, `description?`, `icon?`, `color?` | Update type properties |
| `delete_item_type` | `type_id` | Delete a type (items with this type are not deleted) |
| `list_attribute_kinds` | — | List all attribute kind definitions |
| `get_attribute_by_key` | `key` | Get an attribute kind by its key/slug |
| `create_attribute_kind` | `key`, `base_type`, `description?`, `config?` | Create attribute schema; for `dropdown`, pass `config: {"values": ["opt1", "opt2"]}` |
| `update_attribute_kind` | `kind_id`, `description?`, `config?` | Update attribute config |

Valid `base_type` values: `text`, `integer`, `decimal`, `date`, `week`, `boolean`, `dropdown`, `item`, `list`.

---

## Example prompts and workflows

These examples show the natural-language prompt, the tool call sequence the agent will execute, and what to expect.

### 1. Creating a ticket

**Prompt:** "Create a task called 'Fix login bug' under the Auth project, due this Friday, priority High"

```
search_items("Auth project")
  → finds item_id 42

create_item(title="Fix login bug", content="", attributes={"Date": "2026-06-07"})
  → creates item_id 87

set_item_attributes(id=87, attributes={"Priority": "High"})

assign_item_type(item_id=87, type_name="Task")

# Link to parent (Auth project)
# Note: linking uses the HTTP API directly; the agent may use update_item or
# set_item_attributes to add a parent reference if your schema supports it.
```

### 2. Planning a day

**Prompt:** "Show me today's plan and mark my morning standup habit as complete"

```
get_day_plan("2026-06-05")
  → list of scheduled items

get_repeat_entries("2026-06-05")
  → finds standup habit at item_id 15, status=NotComplete

update_repeat_status(item_id=15, date="2026-06-05", status="Complete")
```

### 3. Reading a project

**Prompt:** "Give me a summary of the Q3 Roadmap project and its open tasks"

```
get_item_by_title("Q3 Roadmap")
  → item_id 33, content, attributes

get_children(id=33)
  → list of child task items

# For each task with Status != "Done":
get_item(id=<task_id>)
  → full details including Status attribute
```

### 4. Updating repeat status with a note

**Prompt:** "I exercised today but only for 20 minutes — log it as alternate with a note"

```
get_repeat_entries("2026-06-05")
  → finds exercise habit at item_id 22

update_repeat_status(
  item_id=22,
  date="2026-06-05",
  status="Alternate",
  comment="Only 20 min today, shortened due to early meeting"
)
```

---

## Safe operating rules for agents

These rules help agents avoid data loss or unintended changes. Paste them into your system prompt or reference this guide when configuring agent behavior.

### Read before writing

Always call `get_item` or `search_items` before calling `update_item`, `set_item_attributes`, or `delete_item`. Confirm the item is correct before changing it.

### Destructive operations require user confirmation

Before executing any of the following, the agent should present what it is about to do and ask the user to confirm:

- `delete_item` — permanently removes the item and all its data
- `delete_comment` — cannot be undone
- `delete_rule` — removes the automation rule
- `delete_media` — permanently removes the file or empty folder
- `delete_item_type` — removes the type definition (items keep their data, but lose the type label)
- `update_attribute_kind` or `delete_attribute_kind` changes affect every item using that attribute

### Attribute updates are merge, not replace

`set_item_attributes` only overwrites the keys you pass. Existing attributes not included in the call are left unchanged. You do not need to read and re-send all attributes to update one field.

### Item content updates are partial

`update_item` updates only the fields you pass. Omitting `content` leaves the existing content unchanged. Omitting `title` leaves the title unchanged.

### Show automation scripts before running

Before calling `create_rule`, `update_rule`, or `run_rule`, show the Lua script to the user and explain what it does. Automation rules run server-side with access to your full Zealot data.

### Schema changes are global

`create_item_type`, `update_item_type`, `delete_item_type`, `create_attribute_kind`, and `update_attribute_kind` affect every item that uses that type or attribute. Treat these like schema migrations — confirm intent before proceeding.

---

## Known limitations

### No export via MCP

Item export to PDF or DOCX is not exposed through MCP tools. Use the HTTP API directly: `GET /item/{id}/export?format=pdf`. See [HTTP API](./api.md).

### No filtered queries

The HTTP API supports rich attribute-based filtering (`GET /item/filter`), but MCP does not expose this endpoint. Work around it with `search_items` (keyword search) or `list_items` with `type_filter`, then filter client-side.

### No batch operations

Each tool call is a single API round-trip. There is no bulk create/update/delete. For large operations, the agent must make multiple sequential calls.

### Event rules do not chain

Automation rules with event triggers (`on_item_create`, `on_type_assign`, etc.) do not re-trigger other event rules. This prevents infinite loops but means you cannot chain event rules. Use a single rule that does everything, or combine event and cron triggers.

### MCP prompts require client support

The server defines 10 structured prompts (`daily_briefing`, `plan_my_week`, `capture_idea`, etc.) that guide the agent through common workflows. These are only available in clients that support the MCP prompts capability. Claude Desktop supports them; Codex CLI and most HTTP clients currently do not. Tools work in all clients regardless.

### HTTP mode compatibility

The HTTP server includes a compatibility layer that converts Server-Sent Events responses to plain JSON for clients that don't handle streaming. This is transparent to the agent but means some clients receive responses slightly later than native SSE clients.

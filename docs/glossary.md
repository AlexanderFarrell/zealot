# Zealot Glossary

---

Definitions for the core terms used throughout Zealot's documentation, UI, and API. Terms are listed alphabetically. Where the canonical definition lives in another document, a cross-reference is provided.

---

### admonition

A styled callout block in ZealotScript, created with `:::kind … :::` fenced syntax. Supported kinds include `note`, `warning`, `tip`, `danger`, `info`, `success`, `important`, `caution`, `example`, `faq`, and `todo`. See [ZealotScript — Admonition kinds](./data-model.md#admonition-kinds).

### API key

A secret token used to authenticate HTTP requests to the Zealot backend, replacing session cookies for scripted or programmatic access. Generated in account settings or via `POST /auth/api_key`. See [HTTP API — Authentication](./http-api.md#authentication).

### attribute

A typed key-value pair stored on an item, such as `Status = "In Progress"` or `Date = "2026-06-05"`. The available keys and their types are defined by attribute kinds. See [Data Model — Attributes](./data-model.md#attributes).

### attribute kind

The schema definition for an attribute: its key name, base type, optional constraints (such as a list of allowed dropdown values), and whether it is system-created. Attribute kinds are shared across all items. See [Data Model — Attributes](./data-model.md#attributes).

### base type

The underlying data type of an attribute kind. One of: `text`, `integer`, `decimal`, `date`, `week`, `dropdown`, `boolean`, `item`, `list`. See [Data Model — Base types](./data-model.md#base-types).

### comment

A timestamped note attached to an item, stored in ZealotScript. Used for journal entries, activity logs, and daily notes. See [Data Model — Comments](./data-model.md#comments).

### cron expression

A five-field time pattern (`minute hour dom month dow`) used to schedule rules. See [Rules Engine — Cron schedule syntax](./rules-engine.md#cron-schedule-syntax).

### Daily Plan

A conventional item type used to represent a day's scratchpad or agenda note. An item of this type typically carries a `Date` attribute so it appears in the daily planner view. This is not a built-in system type — users create and name it as they choose. See [Quickstart](./quickstart.md).

### dropdown

An attribute base type whose value must be one of a predefined set of strings, configured in the attribute kind's `values` array. See [Data Model — Base types](./data-model.md#base-types).

### item

The universal unit of data in Zealot. Every note, task, goal, project, meeting, or habit is an item. An item has a title, a freeform body in ZealotScript, typed attributes, links to other items, and zero or more item types. See [Data Model — Items](./data-model.md#items).

### item type

A user-defined (or system-defined) category that can be assigned to items, such as Task, Project, or Meeting. An item can have zero or more types simultaneously. Types may specify required attribute keys that must be present on any item of that type. Also referred to simply as "type" in context. See [Data Model — Item Types](./data-model.md#item-types).

### link

A directed edge from one item to another, carrying a `relationship` label. Links form a general directed graph — not a strict tree. See [Data Model — Links and Relationships](./data-model.md#links-and-relationships).

### Lua

The scripting language (Lua 5.4) used in the rules engine. Scripts run in a sandboxed environment with access to the `zealot` Lua API. See [Rules Engine](./rules-engine.md).

### MCP

Model Context Protocol. An open standard for connecting AI agents (such as Claude or Codex) to external tools. Zealot ships an MCP server that exposes its data model to agents as a set of typed tools. See [MCP Guide](./mcp.md).

### planner

The date-based views in Zealot (daily, weekly, monthly, annual) that surface items carrying a `Date` or `Week` attribute. The planner is not a separate data entity — it is a view over items. See [Data Model — Planner Entries](./data-model.md#planner-entries).

### planner entry

An item that appears in a planner view because it carries a `Date` attribute (for daily and monthly views) or a `Week` attribute (for weekly and annual views). There is no separate database entity for planner entries. See [Data Model — Planner Entries](./data-model.md#planner-entries).

### relationship

The label on a link between two items. Well-known values are `parent`, `blocks`, `tag`, `topic`, and `other`; any lowercase string is valid as a user-defined relationship. See [Data Model — Links and Relationships](./data-model.md#links-and-relationships).

### repeat

A per-day tracking record for an item that recurs — a habit or routine. A repeat entry captures whether the item was completed, skipped, alternated, or not completed for a specific date. See [Data Model — Repeats](./data-model.md#repeats).

### rule

A Lua 5.4 script paired with a trigger that runs automatically in response to events or on a schedule. Rules can read and write items, set attributes, add comments, and generate content. See [Data Model — Rules](./data-model.md#rules) and [Rules Engine](./rules-engine.md).

### rules engine

The Zealot subsystem that evaluates and executes Lua automation rules in a sandboxed environment. See [Rules Engine](./rules-engine.md).

### session

A browser-based authentication state managed via `session_id` and CSRF cookies. Scripts and API clients should use API keys rather than sessions. See [HTTP API — Authentication](./http-api.md#authentication).

### Statistic

An ordinary item assigned the system Statistic type and configured with Value Kind, Unit, and
Daily Aggregation. It defines one typed numeric series. See [Data Model — Statistics](./data-model.md#statistics).

### Statistic Entry

One timestamped numeric observation belonging to a Statistic, with an optional related item and
comment. See [Data Model — Statistics](./data-model.md#statistics).

### trigger

The condition that causes a rule to run. Triggers can be event-based (for example `on_item_create`, `on_attribute_set`), scheduled (cron or interval), or manual. See [Data Model — Trigger shapes](./data-model.md#trigger-shapes).

### wiki link

A `[[Item Title]]` reference inside ZealotScript content that resolves at render time to a clickable link to the item with that exact title (case-sensitive). `[[type:TypeName]]` links to the item list filtered by that type. See [Data Model — Wiki links](./data-model.md#wiki-links).

### ZealotScript

Zealot's markup language: a superset of GitHub-Flavoured Markdown (GFM) extended with `:::keyword … :::` fenced blocks for admonitions, math, YouTube embeds, and wiki links. Used in item bodies and comment content. See [Data Model — ZealotScript](./data-model.md#zealotscript) and [ZealotScript User Guide](./zealotscript/README.md).

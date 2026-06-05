# Zealot Data Model & Content Syntax

This document is the canonical reference for every persistent entity in Zealot and the ZealotScript markup language. It is authoritative for users, developers, API clients (see [HTTP API](./http-api.md)), and MCP agents (see [MCP Guide](./mcp.md)). All other documentation should stay consistent with what is written here. For definitions of terms used throughout this document and the rest of the docs, see the [Glossary](./glossary.md).

---

## Items

An **item** is the universal unit of data in Zealot. Notes, tasks, goals, projects, habits, meetings — everything is an item.

| Field | Type | Description |
|---|---|---|
| `item_id` | integer | Stable numeric identifier. Use this for all API calls and links. |
| `title` | string | Short display name. Must not be empty. |
| `content` | string | Body text in [ZealotScript](#zealotscript). May be empty. |
| `attributes` | object | Key→value map of typed metadata (see [Attributes](#attributes)). |
| `types` | array of `{type_id, name, is_system}` | Zero or more item types assigned to this item. |
| `links` | array of `{other_item_id, relationship}` | Directed edges to other items (see [Links](#links-and-relationships)). |

### Display title

If the item has an `Icon` attribute (type `text`) its value is prepended to the title for display: `"📌 My Task"`. The stored `title` field is always the bare string without the icon.

### JSON shape (API response)

```json
{
  "item_id": 42,
  "title": "Quarterly review",
  "content": "## Agenda\n\n- Review OKRs\n- Plan next quarter",
  "attributes": {
    "Status": "In Progress",
    "Date": "2026-06-10",
    "Priority": "High"
  },
  "types": [
    { "type_id": 3, "name": "Meeting", "is_system": false }
  ],
  "links": [
    { "other_item_id": 7, "relationship": "parent" }
  ]
}
```

---

## Item Types

**Item types** are user-defined (or system-defined) categories that can be assigned to items. They optionally enforce the presence of specific attribute keys.

| Field | Type | Description |
|---|---|---|
| `type_id` | integer | Stable numeric identifier. |
| `name` | string | Human-readable label, e.g. `"Task"`, `"Goal"`, `"Meeting"`. |
| `description` | string | Free-text description shown in the UI. |
| `required_attributes` | array of strings | Attribute keys that must be present on any item of this type. Validated by `is_valid()`. |
| `is_system` | boolean | `true` for types created by the system; `false` for user-created types. System types cannot be deleted. |

### Multi-type items

An item can have **zero, one, or many** types simultaneously. Types are stored as a list on the item; there is no single "primary" type. Each assigned type's `required_attributes` is checked independently — an item is considered valid for a type when all of that type's required attribute keys are present.

### Required attributes enforcement

Required attributes are enforced at the application layer. If a required key is missing the item still saves, but the UI may flag it as invalid for that type. Use `GET /item_type/` to retrieve the current type definitions before creating or updating items programmatically.

### JSON shape (API response)

```json
{
  "type_id": 3,
  "name": "Task",
  "description": "A unit of work to be completed.",
  "required_attributes": ["Status", "Date"],
  "is_system": false
}
```

---

## Attributes

**Attributes** are typed key→value pairs stored on items. The set of available attribute keys and their types are defined globally as **attribute kinds**. All attribute keys are user-configured — there are no hard-coded required keys except for the special `Icon` convention described in the Items section.

### Attribute kinds

Each attribute kind defines:

| Field | Type | Description |
|---|---|---|
| `kind_id` | integer | Identifier. |
| `key` | string | The attribute key as it appears on items, e.g. `"Status"`, `"Due Date"`. |
| `description` | string | Shown in the UI. |
| `base_type` | string | One of the base types below. |
| `config` | object | Type-specific configuration (constraints, allowed values). |
| `is_system` | boolean | System attribute kinds cannot be deleted. |

Use `GET /attribute/` to retrieve all defined attribute kinds for the instance.

### Base types

| Base type | Wire format | Example value | Config options |
|---|---|---|---|
| `text` | JSON string | `"In Progress"` | `min_len`, `max_len`, `pattern` (regex) |
| `integer` | JSON number (whole) | `42` | `min`, `max` |
| `decimal` | JSON number (float) | `3.14` | `min`, `max` |
| `date` | `"YYYY-MM-DD"` string | `"2026-06-10"` | — |
| `week` | `"YYYY-Www"` string | `"2026-W24"` | — |
| `dropdown` | JSON string (must be one of `values`) | `"High"` | `values` (array of allowed strings) |
| `boolean` | JSON boolean | `true` | — |
| `item` | JSON integer (item_id) | `7` | — |
| `list` | JSON array of any scalar type above | `["a", "b"]` | `list_type` (one of the scalar base types above) |

**Date format:** `YYYY-MM-DD` (e.g. `"2026-06-10"`). Always pass as a string.

**Week format:** `YYYY-Www` with a two-digit week number (e.g. `"2026-W24"`). Weeks follow ISO 8601 (Monday is the first day). Always pass as a string.

**List type:** A list attribute stores an array of values all of the same scalar type. The scalar validation rules (min, max, etc.) apply to each element.

### Attribute config object (per type)

```json
// text
{ "min_len": 1, "max_len": 100, "pattern": "^[A-Z].*" }

// integer / decimal
{ "min": 0, "max": 100 }

// dropdown
{ "values": ["Low", "Medium", "High", "Critical"] }

// list
{ "list_type": "text" }
```

Omitted config fields mean no constraint. For `date`, `week`, `boolean`, and `item` the config object is empty `{}`.

### Filtering items by attribute

`POST /item/filter` accepts an array of attribute filters:

```json
[
  { "key": "Status",   "op": "eq",  "value": "In Progress", "list_mode": "any" },
  { "key": "Priority", "op": "gte", "value": "High",        "list_mode": "any" }
]
```

| Field | Values |
|---|---|
| `op` | `eq`, `=` · `ne`, `!=`, `<>` · `gt`, `>` · `lt`, `<` · `gte`, `>=` · `lte`, `<=` · `ilike` |
| `list_mode` | `any` (default) · `all` · `none` — controls how list attributes are matched |

`ilike` performs a case-insensitive substring match (useful for `text` attributes).

---

## Links and Relationships

Items form a **directed graph**. Each edge is an `ItemLink`:

```json
{ "other_item_id": 7, "relationship": "parent" }
```

The `relationship` field is a lowercase string. Well-known values:

| Relationship | Meaning |
|---|---|
| `parent` | This item's parent in the hierarchy. An item may have multiple parents (DAG). |
| `blocks` | This item blocks the linked item. |
| `tag` | This item is tagged with the linked item (the linked item is the tag). |
| `topic` | This item belongs to the linked topic. |
| `other` | Generic/unclassified relationship. |

Any other lowercase string is valid as a user-defined relationship.

### Navigating the graph

| Goal | API endpoint |
|---|---|
| Get root items (no parent links) | `GET /item/` |
| Get children of an item | `GET /item/children/{id}` |
| Get all linked items (any relationship) | `GET /item/related/{id}` |
| Get items linked via a specific relationship | Use `GET /item/related/{id}` and filter client-side by `relationship` |

**Children:** An item `C` is a child of item `P` if `C` has a link with `relationship == "parent"` and `other_item_id == P`. The `/item/children/{id}` endpoint performs this lookup server-side.

**Topics:** Assign an item to a topic by adding a link with `relationship == "topic"` pointing at the topic item. Any item can serve as a topic.

**Related items:** `GET /item/related/{id}` returns all items that share any link with the given item, regardless of direction or relationship type.

---

## Planner Entries

There is no separate "planner entry" entity. Items **appear on the planner** when they carry a `Date` or `Week` attribute.

| Attribute key | Type | Planner view |
|---|---|---|
| `Date` | `date` — `"YYYY-MM-DD"` | Daily and monthly views |
| `Week` | `week` — `"YYYY-Www"` | Weekly and annual views |

The attribute keys `Date` and `Week` are conventional — they are user-created attribute kinds with these names. Check `GET /attribute/` to confirm they exist in your instance.

### Planner endpoints

| Endpoint | Returns |
|---|---|
| `GET /planner/day/{YYYY-MM-DD}` | Items with a `Date` matching that day |
| `GET /planner/week/{YYYY-Www}` | Items with a `Week` matching that ISO week |
| `GET /planner/month/{M}/year/{YYYY}` | Items with a `Date` in that calendar month |
| `GET /planner/year/{YYYY}` | Items with a `Date` or `Week` in that year |

All planner endpoints return `ItemDto[]` — the same full item objects as the item API.

---

## Repeats

**Repeats** are per-day tracking records for items that recur (habits, routines). A repeat entry records whether a particular item was completed, skipped, or otherwise acted on for a specific date.

### RepeatEntry fields

| Field | Type | Description |
|---|---|---|
| `item` | `ItemDto` | The full item this entry belongs to. |
| `date` | `"YYYY-MM-DD"` | The calendar date for this entry. |
| `status` | string | One of the statuses below. |
| `comment` | string | Optional free-text note for this day. |

### RepeatStatus values

| Wire string | Meaning |
|---|---|
| `"Complete"` | Done for this day. |
| `"Skip"` | Intentionally skipped. |
| `"Alternate"` | Completed an alternate version. |
| `"Not Complete"` | Not done (default). |

### API

```
GET  /repeat/?item_id={id}&date={YYYY-MM-DD}   — get entries
PATCH /repeat/                                  — update status/comment
```

Update body:

```json
{
  "item_id": 42,
  "date": "2026-06-10",
  "status": "Complete",
  "comment": "Did 30 minutes"
}
```

---

## Comments

**Comments** are timestamped notes attached to an item. They are typically used for journal entries, daily logs, or activity history.

| Field | Type | Description |
|---|---|---|
| `comment_id` | integer | Identifier. |
| `item` | `ItemDto` | The full item this comment belongs to. |
| `timestamp` | `"YYYY-MM-DD HH:MM:SS"` | When the comment was recorded. |
| `content` | string | Body in [ZealotScript](#zealotscript). |

### API

```
GET    /comment/?item_id={id}   — list comments for an item
POST   /comment/                — add a comment
PATCH  /comment/{id}            — update
DELETE /comment/{id}            — delete
```

Add body:

```json
{
  "item_id": 42,
  "timestamp": "2026-06-10 09:30:00",
  "content": "Had a productive session today."
}
```

---

## Rules

**Rules** are Lua 5.4 scripts that run automatically in response to events or on a schedule. See the [Rules Engine guide](./rules-engine.md) for the full API reference and cookbook.

### Rule fields

| Field | Type | Description |
|---|---|---|
| `rule_id` | integer | Identifier. |
| `name` | string | Display name. |
| `description` | string | Free-text description. |
| `trigger` | object | When the rule runs (shapes below). |
| `script` | string | Lua 5.4 source code. |
| `enabled` | boolean | Disabled rules do not run. |
| `last_run_at` | datetime or null | When the rule last executed. |
| `last_error` | string or null | Error message from the last run, if any. |
| `last_output` | string or null | Output captured from `zealot.notify()` calls. |

### Trigger shapes

Pass the `trigger` field as a tagged JSON object with a `kind` string:

| `kind` | Extra fields | Fires when |
|---|---|---|
| `"manual"` | — | Only via `POST /rule/{id}/run` |
| `"cron"` | `"expression": "0 8 * * *"` | On a cron schedule (5-field: min hour dom month dow) |
| `"interval"` | `"seconds": 3600` | Every N seconds |
| `"on_item_create"` | — | Any item is created |
| `"on_item_update"` | — | Any item is updated |
| `"on_item_delete"` | — | Any item is deleted |
| `"on_comment_add"` | — | Any comment is added |
| `"on_type_assign"` | `"type_name": "Goal"` (or `null` for any) | A type is assigned to an item |
| `"on_type_unassign"` | `"type_name": "Goal"` (or `null` for any) | A type is unassigned from an item |
| `"on_attribute_set"` | `"attribute_key": "Status"` (or `null` for any) | An attribute is set on an item |

Examples:

```json
{ "kind": "manual" }
{ "kind": "cron", "expression": "0 9 * * 1" }
{ "kind": "on_attribute_set", "attribute_key": "Status" }
{ "kind": "on_type_assign", "type_name": null }
```

### Lua API (summary)

```lua
-- Items
zealot.items.get(id)                          -- item or nil
zealot.items.get_by_title(title)              -- item or nil
zealot.items.find_by_type(type_name)          -- array
zealot.items.search(term)                     -- array
zealot.items.filter({ {key,op,value}, ... })  -- array
zealot.items.create(title, content, opts)     -- opts: { types=[...] }
zealot.items.update(id, { title, content })
zealot.items.set_attribute(id, key, value)
zealot.items.assign_type(id, type_name)
zealot.items.unassign_type(id, type_name)
zealot.items.delete(id)

-- Comments
zealot.comments.add(item_id, content)

-- Utilities
zealot.notify(msg)    -- captured in last_output
zealot.now            -- ISO datetime string
zealot.date           -- "YYYY-MM-DD" today

-- Event context (event-triggered rules only)
zealot.event.kind
zealot.event.item          -- full item (most events)
zealot.event.item_id       -- on_item_delete, on_comment_add
zealot.event.type_name     -- on_type_assign / unassign
zealot.event.attribute_key -- on_attribute_set
```

---

## ZealotScript

**ZealotScript** is the markup language used in item `content` and comment `content` fields. It is a superset of GitHub-Flavored Markdown (GFM), extended with fenced-block syntax (`:::keyword … :::`) for richer content.

### Syntax reference

| Feature | Syntax | Notes |
|---|---|---|
| Heading 1–6 | `# H1` … `###### H6` | |
| Bold | `**text**` | |
| Italic | `*text*` | |
| Strikethrough | `~~text~~` | |
| Underline | `_text_` | |
| Inline code | `` `code` `` | |
| Highlight | `<mark>text</mark>` | HTML tag |
| Subscript | `<sub>text</sub>` | HTML tag |
| Superscript | `<sup>text</sup>` | HTML tag |
| Hard line break | `<br>` | HTML tag |
| External link | `[label](url)` | Standard Markdown |
| Wiki link | `[[Item Name]]` | Navigates to the item with that exact title |
| Type link | `[[type:TypeName]]` | Navigates to the type list filtered to that type |
| Blockquote | `> text` | |
| Bullet list | `- item` | |
| Ordered list | `1. item` | |
| Code block | ` ```lang … ``` ` | Syntax highlighting |
| Table | Standard Markdown pipes | |
| Admonition | `:::note … :::` | See admonition kinds below |
| YouTube embed | `:::youtube VIDEO_ID` | Single-line; no closing `:::` needed |
| Inline math | `$E = mc^2$` | LaTeX via KaTeX |
| Math block | `:::math … :::` | Multi-line LaTeX |
| Emoji | `:smile:` | Expands to the corresponding Unicode character |

### Admonition kinds

Admonitions are styled callout blocks:

```
:::note
This is a note.
:::
```

Supported kinds: `note` · `warning` · `danger` · `tip` · `info` · `success` · `important` · `caution` · `example` · `faq` · `todo`

### Wiki links

`[[Item Name]]` is resolved at render time by looking up an item with that exact title (case-sensitive). If no matching item exists the link is rendered as plain text. This syntax is valid in both the UI editor and when writing content via the API or Lua rules.

`[[type:TypeName]]` links to the item type list filtered to that type name.

### Unsupported syntax

ZealotScript does not support:

- Raw `<script>`, `<style>`, `<iframe>`, or other arbitrary HTML beyond the specific tags listed above (`<mark>`, `<sub>`, `<sup>`, `<br>`)
- JSX / React component syntax
- Obsidian-style `![[transclusion]]` (planned but not yet implemented)
- Mermaid diagrams (planned but not yet implemented)

Standard GFM constructs (headings, lists, tables, fenced code blocks, blockquotes, bold, italic, strikethrough, external links) are fully supported.

---

## Entity Relationship Summary

```
Account
  └─ owns → Items (many)
  └─ owns → Rules (many)
  └─ owns → AttributeKinds (many)
  └─ owns → ItemTypes (many)

Item
  ├─ has → Attributes (key→value, typed by AttributeKind)
  ├─ assigned → ItemTypes (many-to-many)
  ├─ links to → Items (directed graph via ItemLink.relationship)
  │              well-known: parent | blocks | tag | topic | other
  ├─ has → Comments (many, timestamped ZealotScript notes)
  └─ has → RepeatEntries (one per date tracked)

Planner (not a separate entity)
  └─ Items with a "Date" or "Week" attribute surface in planner views

Rule
  ├─ has → Trigger (event-based | scheduled | manual)
  └─ runs → Lua script that can read/write Items and Comments
```

All IDs are stable integers. The item graph supports cycles and multiple parents (it is a general directed graph, not a strict tree).

# Zealot HTTP API Reference

This document covers every HTTP endpoint exposed by the Zealot backend. It is intended for script authors, client developers, and anyone who wants to interact with Zealot programmatically without reading the server source.

---

## Overview

| Property | Value |
|---|---|
| Default base URL | `http://localhost:8456` |
| Content-Type (requests) | `application/json` |
| Content-Type (responses) | `application/json` (except exports and file downloads) |

All endpoints under `/item`, `/item_type`, `/attribute`, `/comment`, `/planner`, `/repeat`, `/rule`, `/media`, and `/account` require authentication. The health endpoints and `POST /auth/api_key` are public.

---

## Why use the API?

The HTTP API exposes the full Zealot data model over a standard REST interface. Every operation available in the web UI is also available programmatically — which opens up a range of workflows the UI alone cannot support.

- **Script your workflow.** Any shell script, Python script, or HTTP client can create items, set attributes, and read the planner. A cron job that fetches today's scheduled items and prints them to your terminal takes about ten lines of shell.
- **Integrate with other tools.** Pull data in from external sources (GitHub issues become Zealot tasks; calendar events appear on the planner) or push Zealot data out (export completed tasks to a spreadsheet, trigger a notification when a goal's status changes).
- **Build your own clients.** The official web and mobile apps communicate with Zealot exclusively through this API. You can build a thin CLI, a status-bar widget, a mobile shortcut, or a custom dashboard using the same interface.
- **Automate beyond the rules engine.** The built-in rules engine runs Lua on the server. The HTTP API lets you drive the same logic from any language, any machine, and any trigger — a push notification, a webhook, a scheduled GitHub Action.

A minimal example — fetch today's plan from a shell script:

```bash
curl -s -H "x-api-key: zlt_abc123" \
  "http://localhost:8456/planner/day/$(date +%Y-%m-%d)" | jq '.[].title'
```

---

## Authentication

Zealot supports two authentication methods. **API keys are preferred for scripts** because they bypass session cookie and CSRF requirements.

### API key (recommended for scripts)

Pass the raw key in an HTTP header:

```
x-api-key: zlt_abc123...
```

No cookies or CSRF tokens are needed. Every endpoint that accepts a session also accepts an API key.

### Session (browser / interactive use)

Session-based auth uses two cookies set by the server on login:

| Cookie | HTTP-only | Purpose |
|---|---|---|
| `session_id` | yes | Authenticates the session |
| `csfr_` | no | CSRF token (readable by JS) |

Mutating requests (`POST`, `PUT`, `PATCH`, `DELETE`) made with a session must also include:

```
x-csrf-token: <value of the csfr_ cookie>
```

If the tokens do not match the server returns `403 Forbidden`.

### Getting an API key

**Option 1 — single request (no prior session needed):**

```
POST /auth/api_key
```

```json
{
  "username": "alice",
  "password": "hunter2",
  "label": "My script"
}
```

Response:

```json
{
  "key": "zlt_abc123...",
  "api_key_id": 7,
  "label": "My script",
  "created_at": "2026-06-05 10:00:00"
}
```

Save the `key` immediately — it is shown only once.

**Option 2 — via the account API (requires an active session):**

```
POST /account/api-keys
{"label": "My script"}
```

Returns the same `CreateApiKeyResponseDto` shape above.

---

## Error shapes

All errors return a plain-text body (not JSON).

| Status | Body | When |
|---|---|---|
| 400 | error message string | Invalid input (validation failure, bad filter op, etc.) |
| 401 | `Unauthorized` | Not authenticated, or resource belongs to another account |
| 403 | `Forbidden` | CSRF token missing or mismatched |
| 404 | `Not found` | Resource does not exist |
| 409 | error message string | Resource is in use and cannot be deleted (pass `?force=true` to override) |
| 500 | `Internal error` | Unexpected server error |

---

## Common types

### ItemDto

Returned by all item endpoints.

```json
{
  "item_id": 42,
  "title": "My item",
  "content": "Body text in ZealotScript",
  "attributes": {
    "Status": "In Progress",
    "Due Date": "2026-06-30"
  },
  "types": [
    { "type_id": 3, "is_system": false, "name": "Task" }
  ],
  "links": [
    { "other_item_id": 10, "relationship": "parent" }
  ]
}
```

### Attribute wire formats

Attributes are stored as typed values. The JSON representation on the wire:

| Base type | JSON type | Example |
|---|---|---|
| `text` | string | `"hello"` |
| `integer` | number (i64) | `42` |
| `decimal` | number (f64) | `3.14` |
| `date` | string | `"2026-06-05"` |
| `week` | string | `"2026-W23"` |
| `dropdown` | string (must be a configured value) | `"High"` |
| `boolean` | boolean | `true` |
| `item` | number (item_id i64) | `99` |
| `list` | array of any scalar above | `[1, 2, 3]` |

Dates use `YYYY-MM-DD`. Weeks use `YYYY-Www` with a zero-padded two-digit week number (e.g. `2026-W03`).

### Item relationships

The `relationship` field in a link is a lowercase string. Well-known values:

| Value | Meaning |
|---|---|
| `parent` | Hierarchical parent (creates the item tree) |
| `blocks` | This item blocks the other |
| `tag` | Tag relationship |
| `topic` | Topic grouping |
| `other` | Generic / unclassified |

Any custom lowercase string is also valid.

---

## Endpoints

### Health

| Method | Path | Auth | Description |
|---|---|---|---|
| GET | `/health/` | No | Returns `"ok"` |
| GET | `/health/ready` | No | Returns `"ready"` |

---

### Auth — `/auth`

| Method | Path | Auth | Description |
|---|---|---|---|
| GET | `/auth/` | No | Check session; refreshes CSRF cookie |
| GET | `/auth/is_logged_in` | No | Same as above |
| POST | `/auth/register` | No | Create account and log in |
| POST | `/auth/login` | No | Log in |
| POST | `/auth/logout` | Session | Log out and clear cookies |
| POST | `/auth/api_key` | No | Get API key in one request using credentials |

All successful auth responses set `session_id` (HTTP-only, 30 days) and `csfr_` (not HTTP-only, 30 days) cookies and return `AccountDto`.

**POST /auth/register**

```json
{
  "username": "alice",
  "password": "hunter2",
  "email": "alice@example.com",
  "given_name": "Alice",
  "surname": "Smith"
}
```

**POST /auth/login**

```json
{
  "username": "alice",
  "password": "hunter2"
}
```

**AccountDto** (returned on login, register, and session check):

```json
{
  "account_id": 1,
  "username": "alice",
  "email": "alice@example.com",
  "given_name": "Alice",
  "surname": "Smith",
  "settings": {},
  "has_api_key": true
}
```

---

### Account — `/account`

All endpoints require authentication.

| Method | Path | Description |
|---|---|---|
| PATCH | `/account/settings` | Update account settings |
| GET | `/account/api-keys` | List API keys |
| POST | `/account/api-keys` | Create API key |
| DELETE | `/account/api-keys/{id}` | Revoke API key |
| GET | `/account/scopes/{scope_id}/members` | List members visible to an authorized scope reader |
| PATCH | `/account/scopes/{scope_id}/members/{principal_id}` | Change a member role (owner/manage-members only) |
| DELETE | `/account/scopes/{scope_id}/members/{principal_id}` | Revoke a member (owner/manage-members only) |
| GET | `/account/scopes/{scope_id}/invitations` | List invitation metadata (owner/manage-members only) |
| POST | `/account/scopes/{scope_id}/invitations` | Create a one-time scope invitation |
| POST | `/account/scopes/{scope_id}/invitations/{invitation_id}/approve` | Approve and activate a pending invitation |
| POST | `/account/scopes/{scope_id}/invitations/{invitation_id}/cancel` | Cancel an unused invitation |
| POST | `/account/scopes/{scope_id}/invitations/{invitation_id}/revoke` | Revoke an unused invitation |
| POST | `/account/invitations/accept` | Accept an invitation as the authenticated account, or begin pending admission |

**PATCH /account/settings** — body is a free-form JSON object stored as account settings.

**POST /account/api-keys**

```json
{ "label": "My script" }
```

Returns `CreateApiKeyResponseDto` (see [Getting an API key](#getting-an-api-key)).

**GET /account/api-keys** — returns an array of `ApiKeyRecordDto` (raw key is not included):

```json
[
  { "api_key_id": 7, "label": "My script", "created_at": "2026-06-05 10:00:00" }
]
```

**DELETE /account/api-keys/{id}** — returns `204 No Content`.

#### Scope invitations and membership

Membership routes authorize the authenticated server principal in the target
scope. Unauthorized target lookups use a privacy-safe `404` response. Session
mutations require the normal `x-csrf-token`; API-key requests do not.

**POST /account/scopes/{scope_id}/invitations** accepts:

```json
{
  "role": "viewer",
  "recipient_principal_id": "optional-principal-uuid",
  "ttl_seconds": 604800
}
```

The response includes invitation metadata and the opaque `token` exactly once;
the server signs the versioned envelope with its server-local invitation key.
The token is not returned by list/read routes and is never a session, API key,
password reset, or replicated credential. `ttl_seconds` must be between five
minutes and thirty days.

**POST /account/invitations/accept** accepts:

```json
{ "token": "z1.<opaque-nonce>.<signature>" }
```

An authenticated account receives an active membership when the invitation and
recipient constraints are valid. An anonymous bearer receives `202 Accepted`
with `pending_admission`; the bearer still needs the server's normal account
enrolment and verification flow. The bearer never becomes a session by itself.

Role changes and revocation take effect through the same scope authorization
used by items, search, planner, comments, media, and service principals. The
final active owner cannot be downgraded or removed.

---

### Items — `/item`

All endpoints require authentication.

| Method | Path | Description |
|---|---|---|
| GET | `/item/` | Get root items (no parent link) |
| GET | `/item/?type=Task` | Get items of a given type |
| POST | `/item/` | Create item |
| GET | `/item/recent` | Get recently modified items |
| GET | `/item/title/{title}` | Get item by exact title |
| GET | `/item/id/{item_id}` | Get item by ID |
| GET | `/item/id/{item_id}/export/pdf` | Export item as PDF |
| GET | `/item/id/{item_id}/export/docx` | Export item as DOCX |
| GET | `/item/search?term=foo` | Search items by title keyword |
| GET | `/item/children/{item_id}` | Get items with a parent link to this item |
| GET | `/item/related/{item_id}` | Get items linked to this item (any relationship) |
| POST | `/item/filter` | Filter items by attribute values |
| POST | `/item/rebuild-links` | Rebuild wiki-link relationships from item bodies |
| PATCH | `/item/{item_id}` | Update item |
| DELETE | `/item/{item_id}` | Delete item |
| PATCH | `/item/{item_id}/attr` | Set (merge) attributes on an item |
| PATCH | `/item/{item_id}/attr/rename` | Rename an attribute key |
| DELETE | `/item/{item_id}/attr/{key}` | Remove an attribute |
| POST | `/item/{item_id}/assign_type/{type_name}` | Assign a type to an item |
| DELETE | `/item/{item_id}/assign_type/{type_name}` | Unassign a type from an item |

#### Pagination

`GET /item/recent` supports:

| Param | Default | Description |
|---|---|---|
| `limit` | 30 | Maximum number of items to return |
| `offset` | 0 | Number of items to skip |

All other listing endpoints return all matching items with no pagination.

#### POST /item/ — create item

```json
{
  "title": "Write homepage copy",
  "content": "Draft content here.\n\nCan use [[wiki links]].",
  "attributes": {
    "Status": "Not Started",
    "Due Date": "2026-06-30"
  },
  "types": ["Task"],
  "links": [
    { "other_item_id": 10, "relationship": "parent" }
  ]
}
```

`attributes`, `types`, and `links` are optional. Returns `ItemDto`.

#### PATCH /item/{item_id} — update item

All fields are optional. `item_id` in the body must match the path parameter.

```json
{
  "item_id": 42,
  "title": "Updated title",
  "content": "Updated body",
  "attributes": { "Status": "In Progress" },
  "links": [{ "other_item_id": 10, "relationship": "parent" }]
}
```

Returns updated `ItemDto`.

#### PATCH /item/{item_id}/attr — merge attributes

Merges the provided key-value pairs into the item's existing attributes. Omitted keys are left unchanged.

```json
{ "Status": "Complete", "Priority": "High" }
```

Returns `200 OK`.

#### PATCH /item/{item_id}/attr/rename

```json
{ "old_key": "Status", "new_key": "state" }
```

Returns `200 OK`.

#### DELETE /item/{item_id}/attr/{key}

Returns `200 OK`.

#### POST /item/filter — filter by attribute

```json
{
  "filters": [
    { "key": "Status", "op": "eq", "value": "In Progress", "list_mode": "any" },
    { "key": "Priority", "op": "eq", "value": "High", "list_mode": "any" }
  ],
  "limit": 50,
  "offset": 0
}
```

Filter operators: `eq` / `=`, `ne` / `!=` / `<>`, `gt` / `>`, `lt` / `<`, `gte` / `>=`, `lte` / `<=`, `ilike`.

`value` may be a scalar or an array of candidate values. `list_mode` applies when the attribute is a list type: `any` (default; any candidate may match), `all` (every candidate must match), `none` (no candidate may match). For scalar attributes use `any`.

`ilike` performs a case-insensitive substring match for text values. `limit` defaults to 50 and is capped at 100. `offset` defaults to 0.

Returns array of `ItemDto`.

#### POST /item/rebuild-links

Scans all item bodies for `[[wiki link]]` syntax and updates the link records in the database. Returns:

```json
{ "rebuilt": 47 }
```

#### Exports

`GET /item/id/{item_id}/export/pdf` and `.../export/docx` return the item rendered as a file download. The `Content-Disposition` header includes the sanitised item title as the filename.

---

### Item Types — `/item_type`

All endpoints require authentication.

| Method | Path | Description |
|---|---|---|
| GET | `/item_type/` | List all types |
| GET | `/item_type/summary` | List types with item counts |
| GET | `/item_type/{type_id}` | Get type by ID |
| GET | `/item_type/name/{name}` | Get type by name |
| POST | `/item_type/` | Create type |
| PATCH | `/item_type/{type_id}` | Update type |
| DELETE | `/item_type/{type_id}` | Delete type |
| POST | `/item_type/{type_id}/attr_kind` | Associate attribute kinds with a type |
| DELETE | `/item_type/{type_id}/attr_kind` | Disassociate attribute kinds |

#### ItemTypeDto

```json
{
  "type_id": 3,
  "is_system": false,
  "name": "Task",
  "description": "A unit of work",
  "required_attributes": ["Status", "Due Date"]
}
```

#### ItemTypeSummaryDto

```json
{
  "type_id": 3,
  "is_system": false,
  "name": "Task",
  "required_attributes_count": 2,
  "item_count": 14
}
```

#### POST /item_type/ — create type

```json
{
  "name": "Task",
  "description": "A unit of work",
  "required_attributes": ["Status"]
}
```

#### PATCH /item_type/{type_id} — update type

All fields optional. `type_id` in the body must match the path parameter.

```json
{
  "type_id": 3,
  "name": "Task",
  "description": "Updated description",
  "required_attributes": ["Status", "Due Date"]
}
```

#### DELETE /item_type/{type_id} — delete type

Returns `200 OK` with no body on success.

If any items are currently assigned this type, returns `409 Conflict` with a message such as:
```
Item type 'Task' is assigned to 37 item(s). Unassign first or pass force=true.
```

Pass `?force=true` to proceed anyway; the cascade constraint removes all type assignments from items (items themselves are not deleted).

System types (`is_system: true`) cannot be deleted and return `400`.

#### POST /item_type/{type_id}/attr_kind — associate attribute kinds

Body is a JSON array of attribute kind keys:

```json
["Status", "Due Date", "Priority"]
```

#### DELETE /item_type/{type_id}/attr_kind

Same shape — array of keys to disassociate.

---

### Attributes — `/attribute`

Attributes here are **attribute kinds** — the schema definitions shared across all items. Individual item attribute values are set via `/item/{id}/attr`.

All endpoints require authentication.

| Method | Path | Description |
|---|---|---|
| GET | `/attribute/` | List all attribute kinds |
| POST | `/attribute/` | Create attribute kind |
| GET | `/attribute/id/{kind_id}` | Get by ID |
| PATCH | `/attribute/id/{kind_id}` | Update by ID |
| GET | `/attribute/key/{key}` | Get by key |
| DELETE | `/attribute/key/{key}` | Delete by key |

#### AttributeKindDto

```json
{
  "kind_id": 5,
  "key": "Status",
  "description": "Current work status",
  "is_system": false,
  "base_type": "dropdown",
  "config": { "values": ["Not Started", "In Progress", "Complete", "Blocked"] }
}
```

#### POST /attribute/ — create attribute kind

The `base_type` string and `config` shape depend on the type:

| base_type | config shape |
|---|---|
| `text` | `{ "min_len": 0, "max_len": 500, "pattern": null }` (all optional) |
| `integer` | `{ "min": null, "max": null }` (optional bounds) |
| `decimal` | `{ "min": null, "max": null }` (optional bounds) |
| `date` | `{}` |
| `week` | `{}` |
| `boolean` | `{}` |
| `item` | `{}` |
| `dropdown` | `{ "values": ["Option A", "Option B"] }` |
| `list` | `{ "list_type": "text" }` (or any other scalar type name) |

```json
{
  "key": "Status",
  "description": "Current work status",
  "base_type": "dropdown",
  "config": { "values": ["Not Started", "In Progress", "Complete", "Blocked"] }
}
```

#### DELETE /attribute/key/{key} — delete attribute kind

Returns `200 OK` with no body on success.

If any items have a value stored for this attribute key, returns `409 Conflict` with a message such as:
```
Attribute kind 'Status' is used by 14 item(s). Delete those values first or pass force=true.
```

Pass `?force=true` to proceed anyway; all per-item attribute values for this key are deleted before the kind is removed.

#### PATCH /attribute/id/{kind_id} — update attribute kind

All fields optional. `kind_id` in the body must match the path parameter.

```json
{
  "kind_id": 5,
  "key": "Status",
  "description": "Updated description",
  "base_type": "dropdown",
  "config": { "values": ["Not Started", "In Progress", "Complete", "Blocked", "Cancelled"] }
}
```

---

### Comments — `/comment`

All endpoints require authentication.

| Method | Path | Description |
|---|---|---|
| GET | `/comment/item/{item_id}` | Get all comments for an item |
| GET | `/comment/day/{date}` | Get comments for a day (`YYYY-MM-DD`) |
| POST | `/comment/` | Add comment |
| PATCH | `/comment/{comment_id}` | Update comment |
| DELETE | `/comment/{comment_id}` | Delete comment |

#### CommentDto

```json
{
  "comment_id": 11,
  "item": { /* ItemDto */ },
  "timestamp": "2026-06-05 09:30:00",
  "content": "This is a comment."
}
```

#### POST /comment/ — add comment

```json
{
  "item_id": 42,
  "timestamp": "2026-06-05 09:30:00",
  "content": "Leaving a note here."
}
```

Timestamp format: `YYYY-MM-DD HH:MM:SS`.

#### PATCH /comment/{comment_id} — update comment

All fields optional. `comment_id` in the body must match the path parameter.

```json
{
  "comment_id": 11,
  "content": "Corrected text.",
  "timestamp": "2026-06-05 09:35:00"
}
```

---

### Planner — `/planner`

Returns items that have an attribute value placing them on the requested time range. All endpoints require authentication and return an array of `ItemDto`.

| Method | Path | Example |
|---|---|---|
| GET | `/planner/day/{date}` | `/planner/day/2026-06-05` |
| GET | `/planner/week/{week}` | `/planner/week/2026-W23` |
| GET | `/planner/month/{month}/year/{year}` | `/planner/month/6/year/2026` |
| GET | `/planner/year/{year}` | `/planner/year/2026` |

Date format: `YYYY-MM-DD`. Week format: `YYYY-Www`. Month is an integer `1`–`12`.

Items appear on the planner when they have a `date` or `week` attribute matching the requested range.

---

### Repeats — `/repeat`

Repeats track completion status for recurring items. All endpoints require authentication.

| Method | Path | Description |
|---|---|---|
| GET | `/repeat/items` | List all items enrolled in the repeat tracker |
| GET | `/repeat/day/{date}` | Get repeat entries for a day (`YYYY-MM-DD`) |
| GET | `/repeat/range` | Get repeat entries across a date range (`?start=YYYY-MM-DD&end=YYYY-MM-DD`) |
| PUT | `/repeat/status` | Update the status of a repeat entry |

#### RepeatEntryDto

```json
{
  "status": "Complete",
  "item": { /* ItemDto */ },
  "date": "2026-06-05",
  "comment": "Done early."
}
```

Status values: `Complete`, `Skip`, `Alternate`, `Not Complete`.

#### GET /repeat/items

Returns `ItemDto[]` — all items that have the `Repeat` type assigned. Useful for building a habits overview or settings screen without needing to guess a date.

#### GET /repeat/range

Query parameters:

| Parameter | Required | Description |
|---|---|---|
| `start` | yes | Start date (`YYYY-MM-DD`, inclusive) |
| `end` | yes | End date (`YYYY-MM-DD`, inclusive) |

Returns `RepeatEntryDto[]` — one entry per item per day in the range on which that item is scheduled. Items that have no `repeat_entry` record for a day default to `Not Complete`. Returns `400` if either date is malformed or if `end` is before `start`.

#### PUT /repeat/status

```json
{
  "item_id": 42,
  "date": "2026-06-05",
  "status": "Complete",
  "comment": "Done early."
}
```

`status` and `comment` are optional. Returns `204 No Content`.

---

### Rules — `/rule`

Rules are Lua scripts that run automatically in response to events or on a schedule. All endpoints require authentication.

| Method | Path | Status | Description |
|---|---|---|---|
| GET | `/rule/` | 200 | List all rules |
| POST | `/rule/` | 201 | Create rule |
| GET | `/rule/{id}` | 200 | Get rule by ID |
| PATCH | `/rule/{id}` | 200 | Update rule |
| DELETE | `/rule/{id}` | 204 | Delete rule |
| POST | `/rule/{id}/run` | 200 | Run rule manually now |

#### RuleDto

```json
{
  "rule_id": 1,
  "account_id": 1,
  "name": "Stamp complete date",
  "description": "Sets a CompletedAt date when Status becomes Complete",
  "trigger": { "kind": "on_attribute_set", "attribute_key": "Status" },
  "script": "-- Lua script here",
  "enabled": true,
  "created_at": "2026-06-01T08:00:00",
  "last_run_at": "2026-06-05T09:00:00",
  "last_error": null,
  "last_output": "stamped item 42"
}
```

#### TriggerKind

The `trigger` object is a tagged union on the `kind` field:

| kind | Extra fields | Description |
|---|---|---|
| `on_item_create` | — | Fires when any item is created |
| `on_item_update` | — | Fires when any item is updated |
| `on_item_delete` | — | Fires when any item is deleted |
| `on_comment_add` | — | Fires when a comment is added |
| `on_type_assign` | `type_name?: string` | Fires when a type is assigned (optionally filtered to one type) |
| `on_type_unassign` | `type_name?: string` | Fires when a type is unassigned |
| `on_attribute_set` | `attribute_key?: string` | Fires when an attribute is set (optionally filtered to one key) |
| `cron` | `expression: string` | Cron schedule expression |
| `interval` | `seconds: number` | Repeats every N seconds |
| `manual` | — | Only runs when triggered explicitly via `POST /rule/{id}/run` |

Examples:

```json
{ "kind": "cron", "expression": "0 9 * * 1-5" }
{ "kind": "interval", "seconds": 3600 }
{ "kind": "on_type_assign", "type_name": "Task" }
{ "kind": "on_attribute_set", "attribute_key": "Status" }
{ "kind": "manual" }
```

#### POST /rule/ — create rule

```json
{
  "name": "Daily summary",
  "description": "Generates a daily plan item at 8am",
  "trigger": { "kind": "cron", "expression": "0 8 * * *" },
  "script": "-- Lua code",
  "enabled": true
}
```

`description` and `enabled` (defaults to `true`) are optional.

#### PATCH /rule/{id} — update rule

All fields optional.

```json
{
  "enabled": false,
  "script": "-- Updated Lua"
}
```

#### POST /rule/{id}/run — run now

No request body. Returns `RuleRunResult`:

```json
{
  "rule_id": 1,
  "success": true,
  "output": "processed 3 items",
  "error": null,
  "duration_ms": 42
}
```

---

### Media — `/media`

File and folder management. All endpoints require authentication.

| Method | Path | Description |
|---|---|---|
| GET | `/media/` | List root directory |
| GET | `/media/*path` | List directory or download file |
| POST | `/media/mkdir` | Create folder |
| POST | `/media/*path` | Upload file (multipart/form-data) |
| PATCH | `/media/rename` | Rename file or folder |
| DELETE | `/media/*path` | Delete file or folder |

#### GET /media/ and GET /media/*path

If the path is a folder, returns a directory listing:

```json
{
  "files": [
    {
      "path": "images/photo.jpg",
      "size": 204800,
      "is_folder": false,
      "modified_at": 1717580400
    },
    {
      "path": "images/archive",
      "size": 0,
      "is_folder": true,
      "modified_at": 1717500000
    }
  ]
}
```

`size` is in bytes. `modified_at` is a Unix timestamp.

If the path is a file, returns the file as a download with:
- `Content-Type` guessed from the file extension
- `Content-Disposition: attachment; filename="..."`
- `ETag: <sha256 of file contents>`

**Conditional GET:** pass `If-None-Match: <etag>` to get `304 Not Modified` if the file has not changed.

#### POST /media/mkdir

```json
{ "folder": "documents/2026" }
```

Returns `200 OK`.

#### POST /media/*path — upload

Send `multipart/form-data` with the file field. The path determines where the file is stored.

```
curl -X POST http://localhost:8456/media/images/photo.jpg \
  -H "x-api-key: $KEY" \
  -F "file=@photo.jpg"
```

#### PATCH /media/rename

```json
{
  "old_location": "images/old-name.jpg",
  "new_name": "new-name.jpg"
}
```

`new_name` is just the filename, not a full path. Returns `200 OK`.

#### DELETE /media/*path

Returns `200 OK`.

---

## curl examples

Set your API key once:

```bash
export BASE=http://localhost:8456
export KEY=zlt_your_key_here
```

### Get an API key

```bash
curl -s -X POST "$BASE/auth/api_key" \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","password":"hunter2","label":"My script"}' \
  | jq .key
```

### Create an item

```bash
curl -s -X POST "$BASE/item/" \
  -H "Content-Type: application/json" \
  -H "x-api-key: $KEY" \
  -d '{
    "title": "Write homepage copy",
    "content": "Draft content here.",
    "types": ["Task"],
    "attributes": {"Status": "Not Started", "Due Date": "2026-06-30"}
  }' | jq .item_id
```

### Get item children

```bash
curl -s "$BASE/item/children/10" \
  -H "x-api-key: $KEY" | jq '.[].title'
```

### Filter items by attribute

```bash
curl -s -X POST "$BASE/item/filter" \
  -H "Content-Type: application/json" \
  -H "x-api-key: $KEY" \
  -d '{
    "filters": [
      {"key": "Status", "op": "eq", "value": "In Progress", "list_mode": "any"}
    ],
    "limit": 50,
    "offset": 0
  }' | jq '.[].title'
```

### Search items by title

```bash
curl -s "$BASE/item/search?term=homepage" \
  -H "x-api-key: $KEY" | jq '.[].title'
```

### Add a comment

```bash
curl -s -X POST "$BASE/comment/" \
  -H "Content-Type: application/json" \
  -H "x-api-key: $KEY" \
  -d '{
    "item_id": 42,
    "timestamp": "2026-06-05 09:30:00",
    "content": "Reviewed and approved."
  }' | jq .comment_id
```

### Create a rule

```bash
curl -s -X POST "$BASE/rule/" \
  -H "Content-Type: application/json" \
  -H "x-api-key: $KEY" \
  -d '{
    "name": "Morning standup",
    "trigger": {"kind": "cron", "expression": "0 9 * * 1-5"},
    "script": "-- your Lua here",
    "enabled": true
  }' | jq .rule_id
```

### Run a rule manually

```bash
curl -s -X POST "$BASE/rule/1/run" \
  -H "x-api-key: $KEY" | jq '{success,output,error,duration_ms}'
```

### Get today's planner items

```bash
curl -s "$BASE/planner/day/$(date +%Y-%m-%d)" \
  -H "x-api-key: $KEY" | jq '.[].title'
```

---

## Statistics

All routes require authentication.

| Method and path | Result |
|---|---|
| GET /statistic/items?parent_id=ID | Statistic items, optionally filtered by Parent |
| GET /statistic/ITEM_ID/entries?start=&end=&limit=&offset= | Newest-first paginated entries |
| GET /statistic/ITEM_ID/daily?start=&end= | UTC daily aggregate points |
| GET /statistic/ITEM_ID/summary?start=&end= | Period count and numeric summary |
| POST /statistic/ITEM_ID/entries | Create an entry |
| PATCH /statistic/entries/ENTRY_ID | Update supplied fields |
| DELETE /statistic/entries/ENTRY_ID | Delete one entry |

Create accepts value plus optional occurred_at, related_item_id, and comment. Patch accepts those
same editable fields; null clears related_item_id or comment. All timestamps are RFC 3339. Range
start is inclusive and end is exclusive. Entry pages contain count, next_offset, and entries.
Page size defaults to 50 and is limited to 100. Empty summaries contain count 0 and null values.

---

## Worked script: create a project with tasks

The following bash script uses `curl` and `jq` to:

1. Get an API key
2. Create a "Website Redesign" project
3. Add three tasks linked to the project as children
4. Read back and print the children

```bash
#!/usr/bin/env bash
set -euo pipefail

BASE="http://localhost:8456"
USERNAME="alice"
PASSWORD="hunter2"

# 1. Get an API key
KEY=$(curl -sf -X POST "$BASE/auth/api_key" \
  -H "Content-Type: application/json" \
  -d "{\"username\":\"$USERNAME\",\"password\":\"$PASSWORD\",\"label\":\"setup-script\"}" \
  | jq -r .key)

echo "API key obtained."

# 2. Create the project item
PROJECT_ID=$(curl -sf -X POST "$BASE/item/" \
  -H "Content-Type: application/json" \
  -H "x-api-key: $KEY" \
  -d '{
    "title": "Website Redesign",
    "content": "Redesign the company website for Q3.",
    "types": ["Project"]
  }' | jq -r .item_id)

echo "Created project: id=$PROJECT_ID"

# 3. Create tasks linked to the project
TASKS=("Write new homepage copy" "Design mockups" "Set up staging environment")

for TASK_TITLE in "${TASKS[@]}"; do
  TASK_ID=$(curl -sf -X POST "$BASE/item/" \
    -H "Content-Type: application/json" \
    -H "x-api-key: $KEY" \
    -d "{
      \"title\": \"$TASK_TITLE\",
      \"content\": \"\",
      \"types\": [\"Task\"],
      \"attributes\": {\"Status\": \"Not Started\"},
      \"links\": [{\"other_item_id\": $PROJECT_ID, \"relationship\": \"parent\"}]
    }" | jq -r .item_id)
  echo "  Created task '$TASK_TITLE': id=$TASK_ID"
done

# 4. Read back and print children of the project
echo ""
echo "Children of project $PROJECT_ID:"
curl -sf "$BASE/item/children/$PROJECT_ID" \
  -H "x-api-key: $KEY" \
  | jq -r '.[] | "  - \(.item_id): \(.title)"'
```

Expected output:

```
API key obtained.
Created project: id=1
  Created task 'Write new homepage copy': id=2
  Created task 'Design mockups': id=3
  Created task 'Set up staging environment': id=4

Children of project 1:
  - 2: Write new homepage copy
  - 3: Design mockups
  - 4: Set up staging environment
```

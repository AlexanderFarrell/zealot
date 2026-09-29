# GAP-021 — Lua rules can add comments but cannot read them

## Problem

`crates/zealot-lua/src/bindings/` exposes only `zealot.comments.add(item_id, content)`.
There is no way to read comments from Lua — no `get_for_item`, no `get_for_day`, no
`update`, no `delete`.

The HTTP API and MCP both expose full comment CRUD plus day-based queries.

## Impact

Rules that use comments as a log or event journal cannot read back what they wrote:
- A weekly summary rule cannot read this week's comments to synthesise them.
- A deduplication rule ("don't add the same reminder twice") cannot check existing
  comments before adding.
- Rules cannot clean up their own generated comments after they're acted on.

The AI-assisted journalling use case described in `docs/overview.md` (read back a
week's comments and summarise) **cannot be done from Lua** — only from MCP.

## Proposed fix

Extend the `zealot.comments` table in `crates/zealot-lua/src/bindings/`:

```lua
zealot.comments.get_for_item(item_id)
-- Returns array of comment tables: { comment_id, item, timestamp, content }

zealot.comments.get_for_day(date)
-- Returns all comments across all items for YYYY-MM-DD
```

Read-only access is the minimum viable fix. `update` and `delete` are lower priority.

## Files to change

- `crates/zealot-lua/src/bindings/` — extend `comments.rs` (or add to existing binding)
- `crates/zealot-lua/src/lib.rs` — re-register updated comments table
- `docs/rules-engine.md` — document new `zealot.comments` read functions

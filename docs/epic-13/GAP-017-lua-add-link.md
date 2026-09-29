# GAP-017 — Lua rules cannot create or remove item links

## Problem

`crates/zealot-lua/src/bindings/items.rs` exposes no `add_link` or `remove_link`
function. Rules can create items and set attributes, but they cannot establish any
graph relationships between items.

The HTTP API fully supports links: `POST /item/` accepts a `links` array, and
`PATCH /item/{id}` can update links. The MCP also has relationship-based navigation
tools. But from Lua, the item graph is write-locked.

## Impact — High severity

Rules that respond to events cannot wire up the item hierarchy:

- An `on_item_create` rule that auto-creates sub-tasks for a new Project cannot
  attach those sub-tasks as children.
- A rule that creates a "Week Review" item cannot link it under an annual "Reviews"
  parent item.
- The `blocks`, `tag`, and `topic` relationships are completely inaccessible from Lua.

Real-world automation that needs graph structure (virtually all meaningful automation)
requires workarounds or is simply impossible.

## Proposed fix

Add two bindings to `crates/zealot-lua/src/bindings/items.rs`:

```lua
zealot.items.add_link(item_id, other_item_id, relationship)
-- Returns true on success
-- relationship is any lowercase string: "parent", "blocks", "tag", "topic", "other"

zealot.items.remove_link(item_id, other_item_id, relationship)
-- Returns true on success
```

These map to `PATCH /item/{item_id}` with an updated links array, or to a dedicated
service method if one is cleaner.

## Files to change

- `crates/zealot-lua/src/bindings/items.rs` — add `add_link` and `remove_link`
- `docs/rules-engine.md` — document the new functions

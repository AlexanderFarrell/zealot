# GAP-018 — Lua rules cannot read child items or related items

## Problem

`crates/zealot-lua/src/bindings/items.rs` has no `get_children()` or `get_related()`
function. Both exist in the HTTP API and as MCP tools, but are unreachable from Lua.

The existing `filter()` function works on attribute values only — it cannot find items
by their link relationships. So there is no workaround for navigating parent-child
hierarchies from within a rule.

## Impact

Rules cannot:
- Iterate a project's child tasks to compute completion percentage.
- Find all items tagged with a specific tag item.
- Walk up the hierarchy to find an item's parent project.
- Process all children when a parent's status changes.

The goal-progress-rollup cookbook example in `docs/rules-engine.md` attempts to use
`filter({ {key="Parent", op="eq", value=goal.id} })` — but "Parent" is a link
relationship, not an attribute, so this filter returns nothing. **The cookbook example
is broken.**

## Proposed fix

Add two bindings to `crates/zealot-lua/src/bindings/items.rs`:

```lua
zealot.items.get_children(item_id)
-- Returns array of item tables (items that have a parent link pointing to item_id)

zealot.items.get_related(item_id)
-- Returns array of item tables (all items linked to item_id in any direction/relationship)
```

Both call the existing service methods used by the HTTP handlers.

## Files to change

- `crates/zealot-lua/src/bindings/items.rs` — add `get_children` and `get_related`
- `docs/rules-engine.md` — document new functions; fix the goal-progress-rollup
  cookbook example which incorrectly uses filter() for parent relationships

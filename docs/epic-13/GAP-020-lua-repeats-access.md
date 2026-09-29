# GAP-020 — Lua rules cannot read or update repeat/habit entries

## Problem

`crates/zealot-lua/src/bindings/` has no repeats module. Rules cannot query habit
completion status for any date, nor update it.

The HTTP API exposes `GET /repeat/day/{date}` and `PUT /repeat/status`. MCP exposes
`get_repeat_entries` and `update_repeat_status`. But from Lua, the repeat system is
completely invisible.

## Impact

Rules cannot:
- Auto-mark a habit complete when another item event occurs (e.g., mark "Journalled"
  complete when a comment is added to the journal item).
- Build a Sunday evening habit report by reading that week's completion data.
- Reset a streak counter when a habit is marked "Not Complete".
- Compute a completion rate for any habit over any period.

## Proposed fix

Add a `zealot.repeats` table to `crates/zealot-lua/src/bindings/`:

```lua
zealot.repeats.get_for_day(date)
-- Returns array of repeat entry tables for YYYY-MM-DD
-- Each entry: { item = <item table>, date = "YYYY-MM-DD", status = "Complete"|..., comment = "" }

zealot.repeats.set_status(item_id, date, status, comment?)
-- Updates the repeat entry for item_id on date
-- status: "Complete" | "Skip" | "Alternate" | "Not Complete"
-- Returns true on success
```

These call the existing `RepeatService` methods.

## Files to change

- `crates/zealot-lua/src/bindings/` — add `repeats.rs` module
- `crates/zealot-lua/src/lib.rs` — register the repeats table
- `docs/rules-engine.md` — document `zealot.repeats`

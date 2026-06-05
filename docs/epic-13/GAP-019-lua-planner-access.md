# GAP-019 — Lua rules cannot access planner data

## Problem

`crates/zealot-lua/src/bindings/` has no planner module. Rules have no way to query
what items are scheduled for a given day or week.

The HTTP API exposes full planner views (`GET /planner/day/{date}` etc.) and MCP
exposes `get_day_plan`, `get_week_plan`, `get_month_plan`. Lua rules are the only
client that cannot access this data.

## Impact

Rules cannot:
- Build a "morning briefing" item that includes today's scheduled work.
- Detect that a day is already overloaded before scheduling another item.
- Generate a weekly summary that lists what was planned vs. what was completed.
- Auto-create a daily plan note that references the actual day's items.

The overview.md and rules-engine.md cookbook both include "daily agenda" examples —
but the current Lua API cannot actually read the planner to build such an agenda.
The examples create an item and set a date, but cannot list what else is on that day.

## Proposed fix

Add a `zealot.planner` table to `crates/zealot-lua/src/bindings/`:

```lua
zealot.planner.get_for_day(date)
-- Returns array of item tables scheduled for that YYYY-MM-DD date

zealot.planner.get_for_week(week)
-- Returns array of item tables for that YYYY-Www week
```

These call the existing `PlannerService` methods already used by the HTTP handlers.

## Files to change

- `crates/zealot-lua/src/bindings/` — add `planner.rs` module
- `crates/zealot-lua/src/lib.rs` — register the planner table
- `docs/rules-engine.md` — document `zealot.planner`

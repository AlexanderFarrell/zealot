# GAP-004 — Fix `os.date` usage in overview Lua examples

## Problem

`docs/overview.md` shows two Lua rule examples in the "Automation" section that use
`os.date(...)`:

```lua
-- Example 1: stamp completion date
zealot.items.set_attribute(item.id, "Completed Date", os.date("%Y-%m-%d"))

-- Example 2: create a daily agenda item  
local today = os.date("%Y-%m-%d")
```

However, `rules-engine.md` explicitly states that `os` is blocked in the Lua sandbox:

> `os` — no system calls, no `os.execute`, no `os.time` (use `zealot.now` instead)

The sandbox implementation (`crates/zealot-lua/src/sandbox.rs`) confirms this — the
sandbox only enables `StdLib::TABLE | StdLib::STRING | StdLib::MATH | StdLib::UTF8`.
The `os` standard library is not included.

These examples would fail at runtime. Anyone following them would get an error.

## Fix

Replace `os.date(...)` with `zealot.date` or `string.sub(zealot.now, 1, 10)` in the
two examples in `docs/overview.md`.

Corrected example 1:
```lua
if item.attributes["Status"] == "Complete" then
  zealot.items.set_attribute(item.id, "Completed Date", zealot.date)
end
```

Corrected example 2:
```lua
local today = zealot.date
local id = zealot.items.create({ title = "Daily Plan — " .. today })
zealot.items.set_attribute(id, "Date", today)
zealot.items.assign_type(id, "Daily Plan")
zealot.notify("Created: Daily Plan — " .. today)
```

Also review the rules-engine cookbook for any `os.date` uses — the "Remind about items
due tomorrow" example uses string manipulation of `zealot.date`, which is correct, but
double-check no cookbook entry slipped through with `os.date`.

## Files to change

- `docs/overview.md` — two examples in the Automation section

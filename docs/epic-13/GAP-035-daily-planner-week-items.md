# GAP-035 — Items scheduled for a week don't appear in the daily planner

## Problem

`crates/zealot-app/src/services/planner.rs` `get_for_day()` only queries items with
a `"Date"` attribute matching the given date. Items with only a `"Week"` attribute
set — meaning they're scheduled "sometime this week" — do not appear in any day view,
even on days that fall within that week.

## Example

- Item "Weekly retrospective" has `Week = "2026-W23"`, no `Date`.
- Week 23 of 2026 is June 1–7.
- Looking at the daily planner for June 3 returns nothing for this item.
- Looking at the weekly planner for `2026-W23` shows it correctly.

Users who schedule work at the week level (common for non-time-sensitive tasks) have
no day-level visibility of that work.

## Impact

The daily planner becomes incomplete for any user who mixes day-level and week-level
scheduling. A user trying to get a complete picture of today's work must check both
the daily and weekly planner views.

## Proposed fix

Update `get_for_day(date, account)` in the planner service to also return items
whose `"Week"` attribute matches the ISO week that contains `date`.

ISO week calculation is straightforward: given `date = 2026-06-03`, the ISO week is
`2026-W23`. A SQL query can derive this with `date_part('week', $1::date)` (Postgres)
or equivalent.

The day-view result set would then be the union of:
1. Items with `Date = date`
2. Items with `Week = iso_week_of(date)`

The same approach applies to `get_for_month()` — items with a matching `Month`/`Year`
attribute could optionally include items with a matching `Date` in that month.

## Files to change

- `crates/zealot-app/src/services/planner.rs` — update `get_for_day()` to union week items
- `crates/zealot-infra/src/repos/postgres/planner_postgres.rs` — update SQL query
- `docs/data-model.md` — update Planner Entries section to describe the combined behaviour

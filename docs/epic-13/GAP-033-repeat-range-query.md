# GAP-033 — No date-range query for repeat entries

## Problem

`crates/zealot-api/src/http/repeat.rs` only supports `GET /repeat/day/{date}` for
reading repeat/habit completion data. There is no range endpoint.

Fetching a week of habit data requires 7 API calls. Fetching a month requires 28-31.

## Impact

- A weekly review rule (or MCP agent) that summarises habit completion must make 7
  separate requests and aggregate the results.
- The habit streak counter cookbook example in `docs/rules-engine.md` has no way to
  validate current streak data — it blindly increments without checking the actual
  completion history.
- The `docs/overview.md` habit-dashboard use case ("tally the completion rate for
  each habit") is impractical via the current API.

## Proposed fix

Add `GET /repeat/range` with start/end date query parameters:

```
GET /repeat/range?start=2026-06-01&end=2026-06-07
```

Returns an array of `RepeatEntryDto` (one per item per day in the range where the
item is scheduled).

**Service change:** Add `get_for_range(start, end, account)` to `RepeatService` that
loops over dates (or uses a single SQL query with `date BETWEEN $1 AND $2`).

## Dependencies

This ticket is a prerequisite for:
- GAP-024 (MCP repeat range tool)
- GAP-020 (Lua repeat access improvement)

## Files to change

- `crates/zealot-api/src/http/repeat.rs` — add range route
- `crates/zealot-app/src/services/repeat.rs` — add range method
- `crates/zealot-infra/src/repos/postgres/repeat_postgres.rs` — add range query
- `docs/http-api.md` — add to Repeats endpoint table

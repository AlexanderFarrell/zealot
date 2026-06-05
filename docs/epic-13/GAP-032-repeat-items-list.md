# GAP-032 — No endpoint to list all items enrolled in the repeat tracker

## Problem

`crates/zealot-api/src/http/repeat.rs` exposes only two endpoints:
- `GET /repeat/day/{date}` — entries for a specific day
- `PUT /repeat/status` — update a single entry

There is no endpoint to list all items that are currently enrolled as repeating
habits. To see what's in the repeat tracker, you must query by date and infer from
what appears.

## Impact

- Users and agents cannot browse their full habits list without picking a specific date.
- The "Habits Dashboard" use case (see `docs/overview.md`) requires knowing all enrolled
  items — this is currently impossible via the API.
- Building a settings screen for managing which items are in the repeat tracker requires
  this endpoint.
- MCP `get_repeat_entries` is date-scoped — agents cannot answer "what habits do I have?"
  without guessing a date.

## Proposed fix

Add `GET /repeat/items` that returns all items currently enrolled in the repeat tracker
— i.e., all items with the `Repeat` type and a `Schedule` attribute set.

Response: `ItemDto[]` (the same full item shape as the item API).

This is essentially a filtered item list (`find_by_type("Repeat")`) with a bit of
service-layer wrapping, not a new database query.

## Files to change

- `crates/zealot-api/src/http/repeat.rs` — add `GET /repeat/items` route
- `crates/zealot-app/src/services/repeat.rs` — add `get_all_repeat_items` method
- `docs/http-api.md` — add to Repeats endpoint table
- `docs/data-model.md` — update Repeats API section

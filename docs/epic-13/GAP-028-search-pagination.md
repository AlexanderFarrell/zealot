# GAP-028 — Search endpoint hard-capped at 20 results with no pagination

## Problem

`crates/zealot-infra/src/repos/postgres/item_postgres.rs` hardcodes `LIMIT 20` in the
search SQL query. The `GET /item/search?term=...` handler accepts no `limit` or
`offset` parameters.

If more than 20 items match a search term, there is no way to retrieve the rest.
The HTTP API docs don't mention this limit, so users who get 20 results don't know
whether there are more.

## Impact

On any reasonably active Zealot instance:
- "meeting" might match 50+ items — user sees only 20, no indication of truncation.
- The MCP `search_items` tool silently returns at most 20 results.
- Programmatic scripts that rely on search for completeness will miss items.

`GET /item/recent` correctly supports `limit`/`offset` — search should too.

## Proposed fix

1. Add `limit` (default 20, max 200) and `offset` (default 0) query params to
   `GET /item/search`.
2. Update the SQL query in `item_postgres.rs` to use `$limit` and `$offset` instead
   of the hardcoded `LIMIT 20`.
3. Update the MCP `search_items` tool to accept and pass `limit`/`offset`.
4. Document in `http-api.md`.

## Files to change

- `crates/zealot-infra/src/repos/postgres/item_postgres.rs` — parameterise the LIMIT
- `crates/zealot-api/src/http/item.rs` — accept query params
- `apps/mcp/src/tools/wiki.rs` — add `limit`/`offset` to `search_items`
- `docs/http-api.md` — document the params and default limit

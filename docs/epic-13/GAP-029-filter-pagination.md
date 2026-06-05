# GAP-029 — `POST /item/filter` returns all matching items with no limit

## Problem

`crates/zealot-infra/src/repos/postgres/item_attribute_value_postgres.rs` runs the
filter query with no `LIMIT` clause. A broad filter (e.g., `Status != "Complete"`)
on a large account fetches every matching item into memory and serialises them all
into a single HTTP response.

This is a future performance cliff. As accounts grow, an unguarded filter could
produce multi-megabyte responses and cause server-side memory pressure.

## Impact (current)

Low on small personal instances. High as a growth concern:
- A filter for all incomplete items on a 10,000-item account returns ~10,000 items.
- The MCP `filter_items` tool (GAP-013) will inherit the same behaviour.
- No pagination means clients must implement client-side slicing.

## Proposed fix

Add optional `limit` and `offset` fields to the `POST /item/filter` request body:

```json
{
  "filters": [...],
  "limit": 100,
  "offset": 0
}
```

Default: no limit (preserve current behaviour for backward compatibility, or set a
generous default of 500). Apply `LIMIT $limit OFFSET $offset` in the SQL query.

## Files to change

- `crates/zealot-infra/src/repos/postgres/item_attribute_value_postgres.rs` — add LIMIT/OFFSET
- `crates/zealot-api/src/http/item.rs` — update `FilterBody` struct
- `docs/http-api.md` — document the optional fields
- `docs/data-model.md` — update the filter section

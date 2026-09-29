# GAP-037 — No comment search and no date-range query

## Problem

`crates/zealot-api/src/http/comment.rs` exposes only:
- `GET /comment/item/{item_id}` — all comments for one item (unlimited, no pagination)
- `GET /comment/day/{date}` — all comments across all items for a single day

Missing:
- Date-range query (e.g., this week's journal)
- Keyword search across comment content
- All-comments listing with pagination

## Impact

**Weekly / period journalling is clunky:**
To read a week's journal entries (comments logged against a "Journal" item), a user
or agent must make 7 calls to `GET /comment/day/` or 1 call to
`GET /comment/item/{journal_id}` and filter client-side by date range.

**No way to surface related comments:**
An agent asked "what did I write about this project last month?" must fetch all
comments for the item (potentially thousands) and filter by date in the client.

**No search:**
"Find all my comments mentioning 'launch date'" requires fetching all comments for
all items, which is impossible without a bulk listing endpoint.

## Proposed fix

**Priority 1 — Date-range query:**

```
GET /comment/range?start=YYYY-MM-DD&end=YYYY-MM-DD
```

Returns all comments across all items with timestamps in the range (inclusive).
This enables week/month journal review in a single call.

**Priority 2 — Keyword search:**

```
GET /comment/search?term=launch+date
```

Returns comments whose content contains the search term (ILIKE). Optionally
scope to a single item with `?item_id=42`.

**Priority 3 — Pagination on `GET /comment/item/{id}`:**

Items with many comments (long-running projects, journals) should paginate:
```
GET /comment/item/{item_id}?limit=50&offset=0
```

## Files to change

- `crates/zealot-api/src/http/comment.rs` — add range and search routes
- `crates/zealot-app/src/services/comment.rs` — add service methods
- `crates/zealot-infra/src/repos/postgres/comment_postgres.rs` — add queries
- `docs/http-api.md` — document new endpoints

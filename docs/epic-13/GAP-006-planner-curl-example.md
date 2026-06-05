# GAP-006 — Fix curl example using `?date=` query parameter for planner

## Problem

In `docs/http-api.md`, the "Why use the API?" section's minimal example uses a query
parameter:

```bash
curl -s -H "x-api-key: zlt_abc123" \
  "http://localhost:8456/planner/day?date=$(date +%Y-%m-%d)" | jq '.items[].title'
```

But the actual planner endpoint is a **path parameter**, not a query parameter:

```
GET /planner/day/{date}
```

For example: `/planner/day/2026-06-05`

The query-parameter form would hit a non-existent route and return a 404. The correct
invocation is:

```bash
curl -s -H "x-api-key: zlt_abc123" \
  "http://localhost:8456/planner/day/$(date +%Y-%m-%d)" | jq '.[].title'
```

Note also that the planner returns an array of `ItemDto` directly (not an object with
`.items`), so the `jq` filter should be `'.[].title'`, not `'.items[].title'`.

## Fix

In `docs/http-api.md`, correct both the URL and the jq filter in the early example.
The full curl section already shows the correct form; only the early motivational example
is wrong.

## Files to change

- `docs/http-api.md` — the minimal example near line 32

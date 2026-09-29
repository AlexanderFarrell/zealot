# GAP-036 — Comments require a client-provided timestamp with no server default

## Problem

`crates/zealot-infra/src/repos/postgres/comment_postgres.rs` inserts the `timestamp`
value passed from the client directly into the database (`INSERT INTO comment (..., time, ...) VALUES (..., $2, ...)`).

The `AddCommentDto` requires the caller to provide a timestamp. If the caller:
- Omits it → validation error or null insertion
- Provides a wrong value → comment appears at the wrong time
- Provides a value in the past/future → comment ordering breaks

The HTTP API docs and MCP tool both require the caller to provide a `YYYY-MM-DD HH:MM:SS`
timestamp. For human users in the browser UI this is fine. But for scripts, MCP agents,
and Lua rules, requiring a correctly-formatted current timestamp is friction that leads
to mistakes.

## Impact

- MCP agents calling `add_comment` must construct and pass a timestamp in the exact
  format expected — one more thing for the agent to get right.
- Lua rules calling `zealot.comments.add()` don't receive a timestamp at all — the
  current binding may be passing an empty string or ignoring it.
- Comments from scripts with slightly wrong clocks appear out of order.

## Proposed fix

Make `timestamp` optional in `AddCommentDto`. If omitted (or null), default to
`NOW()` at the SQL layer:

```sql
INSERT INTO comment (item_id, time, content, account_id)
VALUES ($1, COALESCE($2, NOW()), $3, $4)
```

Or handle it in the service: if `timestamp` is `None`, use the current server time.

The existing behavior (client-provided timestamp) is preserved when a value is given —
useful for importing historical data.

## Files to change

- `crates/zealot-domain/src/comment.rs` — make `timestamp` optional in `AddCommentDto`
- `crates/zealot-app/src/services/comment.rs` — default to now() when None
- `crates/zealot-api/src/http/comment.rs` — allow null timestamp in request
- `docs/http-api.md` — document timestamp as optional, defaulting to server time
- `docs/mcp.md` — update `add_comment` tool reference (timestamp becomes optional)

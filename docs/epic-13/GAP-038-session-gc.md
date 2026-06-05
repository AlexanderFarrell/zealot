# GAP-038 — Expired sessions are never garbage-collected from the database

## Problem

`crates/zealot-infra/src/repos/postgres/session_postgres.rs` validates sessions by
checking `expires_at > now()` in the SELECT query. This correctly rejects expired
sessions at query time — but the expired rows are never deleted.

On a long-running personal instance with regular logins (and session expiry of 30 days),
expired session rows accumulate indefinitely. Every new login adds a row; rows are only
rejected on read, never removed.

## Impact

Low severity for personal instances — the table might grow to thousands of rows over
years, which is negligible. But it is untidy and the same pattern could become a
problem if the session expiry were shorter or if many accounts were active.

More practically: the session table has no index on `expires_at`, so the
`WHERE expires_at > now()` check does a full-table scan on each request as the table
grows.

## Proposed fix

**Option A (recommended): Add an index + startup cleanup**

1. Add a migration: `CREATE INDEX IF NOT EXISTS idx_session_expires_at ON session (expires_at)`.
2. At server startup, after migrations run, delete all expired sessions:
   ```sql
   DELETE FROM session WHERE expires_at < NOW()
   ```

**Option B: Background cleanup task**

Run a cleanup task once daily (e.g., on the scheduler that already runs rules) that
deletes `session WHERE expires_at < NOW()`.

Option A is simpler and sufficient.

## Files to change

- `crates/zealot-infra/migrations/postgres/` — new migration: index on `session.expires_at`
- `crates/zealot-infra/migrations/sqlite/` — equivalent for SQLite
- `crates/zealot-infra/src/repos/mod.rs` (or startup path) — run expired-session cleanup

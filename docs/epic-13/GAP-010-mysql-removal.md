# GAP-010 — Remove stale MySQL references from documentation

## Problem

`docs/architecture.md` (developer guide) lists MySQL as a supported database in the
environment variable table:

> `DATABASE` — `sqlite`, `postgres`, or `mysql`
> `DB_DATABASE`, `DB_USERNAME`, `DB_PASSWORD`, `DB_HOSTNAME`, `DB_PORT` — PostgreSQL / MySQL only

The codebase has a `crates/zealot-infra/src/repos/mysql.rs` file, but
`crates/zealot-infra/src/repos/mod.rs` only dispatches on `"postgres"` and `"sqlite"`.
MySQL is not a working code path — it exists as a stub or dead code.

`docs/deployment.md` correctly does not mention MySQL; the architecture/developer doc
does. This inconsistency could lead a developer to attempt MySQL setup and fail silently.

## Fix

1. Audit `crates/zealot-infra/src/repos/mysql.rs` — if it is dead code with no working
   implementation, either:
   - Delete it and remove all MySQL mentions from docs, OR
   - Add a note that MySQL support is experimental / in progress

2. Remove (or caveat) the MySQL option from the `DATABASE` variable description in
   `docs/architecture.md`.

3. Ensure `docs/deployment.md` and `docs/quickstart.md` only mention the supported
   options (`sqlite`, `postgres`).

## Files to change

- `docs/architecture.md` — DATABASE env var table
- `crates/zealot-infra/src/repos/mysql.rs` — delete or mark as not yet implemented
- `crates/zealot-infra/src/repos/mod.rs` — confirm no MySQL dispatch exists

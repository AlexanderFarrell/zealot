# GAP-012 — Self-service password reset

## Problem

There is no self-service password reset mechanism. Both `deployment.md` and
`troubleshooting.md` state this as a known limitation:

> "No self-service password reset. Forgotten passwords require direct database access
> to update the password hash or delete and re-create the account."

For a personal self-hosted tool this is a real pain. If you forget your password, the
recovery path is: spin up a database container, find the `account` table, compute a
bcrypt hash manually, UPDATE the row. This is not realistic for non-technical users.

## Scope

This is a backend + frontend feature. Given Zealot is single-user / few-user, a
practical v1 is:

**Option A — Admin reset via env-var secret:**
Add a `RESET_SECRET` env var. A `POST /auth/reset-password` endpoint accepts
`{ username, new_password, secret }` and resets the password if the secret matches.
No email required. The operator sets `RESET_SECRET=some-value` at deploy time and
uses it for recovery. Simple and works offline.

**Option B — Email reset flow:**
Requires an SMTP configuration, email delivery, and a token storage mechanism.
Significantly more infrastructure.

**Recommendation: Option A for now.** It solves the recovery problem for a single
self-hosted operator without any external dependencies. Document the reset endpoint
clearly in `troubleshooting.md`.

## Acceptance criteria

- `POST /auth/reset-password` with `{ username, new_password, reset_secret }` changes
  the password for the named account when `reset_secret` matches the `RESET_SECRET`
  env var.
- If `RESET_SECRET` is not configured, the endpoint returns 404 (not exposed at all).
- The new password is bcrypt-hashed before storage.
- `troubleshooting.md` documents the recovery procedure.

## Files to change

- `crates/zealot-app/src/config.rs` — add `reset_secret: Option<String>`
- `crates/zealot-api/src/http/auth.rs` — add `POST /auth/reset-password` handler
- `docs/troubleshooting.md` — document the new recovery path; remove the "requires
  direct database access" caveat
- `docs/deployment.md` — add `RESET_SECRET` to the env var table

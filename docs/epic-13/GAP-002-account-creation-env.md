# GAP-002 — Wire `ACCOUNT_CREATION_ENABLED` into the server binary

## Problem

The example env files (`scripts/.zealot.example.env`) reference
`ACCOUNT_CREATION_ENABLED`, and the developer architecture doc lists it as a supported
variable:

> `ACCOUNT_CREATION_ENABLED` — Disable after first account to prevent open registration

But `crates/zealot-app/src/config.rs` does not read this variable. Setting
`ACCOUNT_CREATION_ENABLED=false` in a compose file has no effect — anyone can still
register a new account by calling `POST /auth/register`.

The deployment doc and troubleshooting doc both note this as a known limitation and
advise using a reverse-proxy deny rule instead. That workaround is fragile and not
obvious.

## Impact

- An operator who deploys Zealot publicly and sets `ACCOUNT_CREATION_ENABLED=false`
  expecting it to work will have an open registration endpoint.
- The docs contradict themselves: the developer guide says the variable exists; the
  deployment guide says it doesn't work.

## Proposed fix

1. Add `account_creation_enabled: bool` to `ZealotConfig` in
   `crates/zealot-app/src/config.rs`, defaulting to `true`.
2. Pass the config into the Axum router state.
3. In `crates/zealot-api/src/http/auth.rs`, check the flag in the `register` handler
   and return `403 Forbidden` (or `404`) if registration is disabled.
4. Remove the known-limitation callout from `deployment.md` and `troubleshooting.md`.
5. Update the env var reference table in the architecture / developer guide.

## Files to change

- `crates/zealot-app/src/config.rs`
- `crates/zealot-api/src/http/auth.rs`
- `crates/zealot-api/src/http/mod.rs` (if config needs threading through)
- `docs/deployment.md` — remove limitation, document the variable
- `docs/troubleshooting.md` — remove limitation note
- `docs/architecture.md` — update env var table

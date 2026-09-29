# GAP-001 — CORS origin configurable via environment variable

## Problem

CORS `allow_origin` is hardcoded to `tauri://localhost` in
`crates/zealot-api/src/http/mod.rs` (line 36). Every documentation page that
mentions production deployment — `deployment.md`, `troubleshooting.md` — warns:

> "There is no environment variable for this. It requires a source change."

This means anyone who deploys Zealot at a real domain (e.g. `https://zealot.example.com`)
must fork the source, change one line, and rebuild — just to use the app in a browser.

This is a hard deployment barrier. It was acceptable while the only supported client was
Tauri, but it blocks all web deployments without code modification.

## Proposed fix

1. Read a `CORS_ORIGIN` (or `ALLOWED_ORIGIN`) environment variable in
   `crates/zealot-app/src/config.rs`, defaulting to `tauri://localhost` to preserve
   existing behaviour.
2. Thread the config value into the Axum CORS layer in `crates/zealot-api/src/http/mod.rs`.
3. Document the new variable in `deployment.md`, `troubleshooting.md`, and the
   architecture reference table.

Acceptance: deploying with `CORS_ORIGIN=https://zealot.example.com` allows browser
requests from that domain with no source change required.

## Files to change

- `crates/zealot-app/src/config.rs` — add `cors_origin: String` field + env read
- `crates/zealot-api/src/http/mod.rs` — use config value instead of string literal
- `apps/server/Dockerfile` — update ENV comment (if any)
- `docs/deployment.md` — document the variable, remove the "requires source change" caveat
- `docs/troubleshooting.md` — update the CORS section

## Notes

- The default value must remain `tauri://localhost` so the Tauri mobile app continues
  to work without any configuration change.
- Consider supporting a comma-separated list of origins for multi-origin setups, but
  a single string is acceptable for v1.

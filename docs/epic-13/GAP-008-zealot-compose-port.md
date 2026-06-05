# GAP-008 — Port mismatch between `zealot-compose.yml` and documentation

## Problem

`docs/deployment.md` states throughout that the web frontend is served on port **8085**:

- Service table: `web | zealot-web | 8085 (→ 80 internally)`
- "The web frontend is served at **http://your-host:8085**."
- The verification checklist uses `http://your-host:8085/`

But `scripts/zealot-compose.yml` (the pre-built image compose file intended for server
deployments) publishes the web port as **8080**:

```yaml
web:
  ports:
    - "8080:80"
```

The root `docker-compose.yml` (used for development / building from source) correctly
uses 8085.

This means:
- Operators following `docs/deployment.md` and using `scripts/zealot-compose.yml` will
  find Zealot at port 8080, not 8085 as documented.
- The verification checklist will fail.

## Fix — Option A (recommended): Align the compose file to the docs

Change `scripts/zealot-compose.yml` to use `8085:80` to match the documented port.

## Fix — Option B: Document the discrepancy

Add a note in `deployment.md` explaining that `zealot-compose.yml` uses 8080 and the
source-build `docker-compose.yml` uses 8085. Update all examples accordingly.

**Option A is preferred** — one canonical port is simpler for operators.

## Files to change

- `scripts/zealot-compose.yml` — change `"8080:80"` → `"8085:80"` (Option A)
- OR `docs/deployment.md` — document both ports (Option B)

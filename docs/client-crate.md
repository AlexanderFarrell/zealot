# `zealot-client` — the shared Rust client crate

`crates/zealot-client` is the one HTTP client for every machine-facing Zealot
client: the CLI (`apps/cli`), the TUI (`apps/tui`), and the MCP server
(`apps/mcp`). If you are building anything that talks to the Zealot API from
Rust, start here.

## What it provides

| Module | Contents |
|---|---|
| `http` | `ZealotClient` — reqwest transport with `X-API-Key` auth: `get`/`post`/`patch`/`put`/`delete`, `get_bytes` (downloads with Content-Disposition capture), `post_multipart` (uploads), `raw` (arbitrary requests) |
| `api::*` | Typed endpoint methods per area (`items`, `planner`, `repeats`, `time_blocks`, `comments`, `rules`, `media`, `item_types`, `attributes`, `account`, `auth`) using `zealot-domain` DTOs |
| `config` | `Config`/`Profile` — `~/.config/zealot/config.toml` with multi-server profiles, env-var precedence, atomic `0600` writes |
| `types` | Client-side DTOs for endpoints whose server shapes aren't exported (`RuleRunResultDto`, `AttributeKindDto`, `MediaEntry`, …) |
| `error` | `ApiError` (with `is_unauthorized()`) and `ConfigError` |

## Authentication model

Headless clients never use cookies or CSRF. The flow (mirroring the desktop app):

1. `api::auth::create_api_key_with_credentials(url, user, pass, label)` →
   `POST /auth/api_key` returns the raw key **once** plus its id
2. Store `{server_url, api_key, api_key_id}` (the CLI does this in a config profile)
3. `ZealotClient::new(url, key)` sends `X-API-Key` on every request
4. On logout, revoke via `client.revoke_api_key(id)`

Connection resolution (`Config::resolve`): `ZEALOT_URL`+`ZEALOT_API_KEY` env vars
win outright; otherwise the selected profile (explicit → `ZEALOT_PROFILE` →
`default_profile` → sole profile).

## Conventions

- **Paths have no trailing slash** (`/comment`, `/rule`, `/item_type`) — the
  deployed axum router 404s `/comment/`. Root media listing is `/media`.
- Path segments holding user text are percent-encoded via `api::seg` /
  `api::media_path`.
- The API surface of truth is `crates/zealot-api/src/http/*`; verify shapes there
  when adding endpoints.
- Everything is async (tokio). The CLI runs a current-thread runtime; the TUI a
  multi-thread one.

## Adding an endpoint

1. Find the handler in `crates/zealot-api/src/http/<area>.rs` for the exact path,
   params, and response type.
2. If the response maps to a `zealot-domain` DTO, use it. If the handler builds
   ad-hoc JSON, add a DTO to `src/types.rs`.
3. Add a method to the matching `src/api/<area>.rs` `impl ZealotClient` block.
4. Cover it with a wiremock test in `tests/api_tests.rs`
   (`cargo test -p zealot-client`).

The MCP server currently uses the shared transport with its own `json!` bodies;
migrate its tools to the typed methods opportunistically.

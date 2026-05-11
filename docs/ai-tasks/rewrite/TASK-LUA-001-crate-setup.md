# TASK-LUA-001: Create `crates/zealot-lua` and add `mlua` to workspace

## Context

The Lua rules engine requires a new crate that depends on `mlua`. To keep compile times low and avoid pulling Lua into every crate, `zealot-lua` is isolated: only `apps/server` links it. All other crates (`zealot-domain`, `zealot-app`, `zealot-infra`, `zealot-api`) stay clean.

## Goal

Scaffold the `crates/zealot-lua` crate and add `mlua` + `croner` to the workspace dependency list.

## Requirements

### Cargo.toml (workspace root)

Add to `[workspace]` members:
```toml
"crates/zealot-lua"
```

Add to `[workspace.dependencies]`:
```toml
mlua   = { version = "0.10", features = ["lua54", "async", "send", "vendored"] }
croner = "2"
```

- `lua54` — use Lua 5.4
- `async` — enables `create_async_function` and `exec_async`
- `send` — requires all Lua values to be `Send` (needed for Tokio multi-thread)
- `vendored` — compiles Lua from source; no system Lua required

### `crates/zealot-lua/Cargo.toml`

```toml
[package]
name = "zealot-lua"
version = "0.1.0"
edition.workspace = true

[dependencies]
mlua          = { workspace = true }
tokio         = { workspace = true }
zealot-domain = { path = "../zealot-domain" }
zealot-app    = { path = "../zealot-app" }
serde_json    = { workspace = true }
chrono        = { workspace = true }
tracing       = { workspace = true }
thiserror     = { workspace = true }
```

### `crates/zealot-lua/src/lib.rs`

Create a minimal stub:
```rust
pub mod runner;
pub mod sandbox;
pub mod bindings;
```

Create empty modules `runner.rs`, `sandbox.rs`, `bindings/mod.rs` so the crate compiles.

### `apps/server/Cargo.toml`

Add:
```toml
zealot-lua = { path = "../../crates/zealot-lua" }
croner     = { workspace = true }
```

## Dependencies

None — this is the foundation task.

## Files to create/modify

- `Cargo.toml` (workspace root) — add member + deps
- `crates/zealot-lua/Cargo.toml` (new)
- `crates/zealot-lua/src/lib.rs` (new)
- `crates/zealot-lua/src/runner.rs` (new, stub)
- `crates/zealot-lua/src/sandbox.rs` (new, stub)
- `crates/zealot-lua/src/bindings/mod.rs` (new, stub)
- `apps/server/Cargo.toml`

## Verification

```bash
cargo check -p zealot-lua
cargo check -p server
```

Both should compile without errors.

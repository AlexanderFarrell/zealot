# TASK-LUA-007: Lua VM sandbox

## Context

Every rule script runs inside a fresh `mlua::Lua` VM. The sandbox must prevent scripts from accessing the file system, network, or OS, and must prevent infinite loops. Safety is enforced with two independent mechanisms: an instruction-count hook (catches CPU infinite loops) and a wall-clock `tokio::time::timeout` (catches async infinite loops).

## Goal

Implement `crates/zealot-lua/src/sandbox.rs` with sandbox setup and execution wrapper.

## Requirements

### `crates/zealot-lua/src/sandbox.rs`

```rust
use mlua::{Lua, LuaOptions, StdLib};
use tokio::time::{timeout, Duration};

const MAX_INSTRUCTIONS: u32 = 10_000_000;
const MAX_WALL_SECONDS: u64 = 10;

/// Create a new sandboxed Lua VM with safe stdlib only.
pub fn new_sandbox() -> mlua::Result<Lua> {
    // Only allow safe standard libraries
    let safe_libs = StdLib::TABLE | StdLib::STRING | StdLib::MATH | StdLib::UTF8;
    let lua = Lua::new_with(safe_libs, LuaOptions::default())?;

    // Set instruction count hook — fires every 1000 instructions
    lua.set_hook(mlua::HookTriggers::every_nth_instruction(1000), |_lua, _debug| {
        // Use a thread-local counter to track total instructions across hook firings
        INSTRUCTION_COUNT.with(|c| {
            let new_count = c.get() + 1000;
            c.set(new_count);
            if new_count >= MAX_INSTRUCTIONS {
                Err(mlua::Error::RuntimeError("Rule execution exceeded instruction limit".to_string()))
            } else {
                Ok(())
            }
        })
    })?;

    Ok(lua)
}

thread_local! {
    static INSTRUCTION_COUNT: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// Reset the instruction counter before each rule run.
pub fn reset_instruction_count() {
    INSTRUCTION_COUNT.with(|c| c.set(0));
}

/// Execute a Lua script string inside the sandbox with wall-clock timeout.
pub async fn execute_script(lua: &Lua, script: &str) -> Result<(), mlua::Error> {
    reset_instruction_count();
    let chunk = lua.load(script);
    timeout(
        Duration::from_secs(MAX_WALL_SECONDS),
        chunk.exec_async(),
    )
    .await
    .map_err(|_| mlua::Error::RuntimeError("Rule execution timed out".to_string()))?
}
```

### What is NOT available to scripts

The sandboxed Lua has **no** access to:
- `io` — file system reads/writes
- `os` — system calls, `os.execute`, `os.exit`, `os.getenv`
- `require` — loading additional Lua modules
- `dofile` / `loadfile` / `loadstring` — dynamic code loading
- `package` — module system
- `debug` — debug library (metatable bypasses)
- `coroutine` — not included (async is handled by mlua's built-in yield)

### What IS available

- `string`, `table`, `math`, `utf8` — standard safe libraries
- `pairs`, `ipairs`, `next`, `select`, `type`, `tostring`, `tonumber` — built-in Lua functions
- `pcall`, `xpcall`, `error`, `assert` — error handling
- `setmetatable`, `getmetatable`, `rawget`, `rawset`, `rawequal` — table manipulation
- `zealot.*` — injected in TASK-LUA-008

## Dependencies

- TASK-LUA-001

## Files to create/modify

- `crates/zealot-lua/src/sandbox.rs`

## Verification

Write a unit test:
```rust
#[tokio::test]
async fn test_infinite_loop_is_killed() {
    let lua = new_sandbox().unwrap();
    let result = execute_script(&lua, "while true do end").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("instruction limit"));
}

#[tokio::test]
async fn test_io_not_available() {
    let lua = new_sandbox().unwrap();
    let result = execute_script(&lua, "io.read()").await;
    assert!(result.is_err());
}
```

```bash
cargo test -p zealot-lua
```

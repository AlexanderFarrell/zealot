use mlua::{Lua, LuaOptions, StdLib};
use tokio::time::{Duration, timeout};

const MAX_WALL_SECONDS: u64 = 10;

/// Create a new sandboxed Lua VM with safe stdlib only.
pub fn new_sandbox() -> mlua::Result<Lua> {
    let safe_libs = StdLib::TABLE | StdLib::STRING | StdLib::MATH | StdLib::UTF8;
    Lua::new_with(safe_libs, LuaOptions::default())
}

/// Execute a Lua script string inside the sandbox with a wall-clock timeout.
///
/// Uses exec_async so that async bindings (zealot.items.*, zealot.comments.*) work.
/// The wall-clock timeout is the primary guard against infinite loops.
pub async fn execute_script(lua: Lua, script: String) -> Result<(), mlua::Error> {
    timeout(
        Duration::from_secs(MAX_WALL_SECONDS),
        lua.load(&script).exec_async(),
    )
    .await
    .map_err(|_| mlua::Error::RuntimeError("Rule execution timed out".to_string()))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_io_not_available() {
        let lua = new_sandbox().unwrap();
        let result = execute_script(lua, "io.read()".to_string()).await;
        assert!(result.is_err());
    }
}

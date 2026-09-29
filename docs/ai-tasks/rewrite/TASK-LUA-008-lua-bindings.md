# TASK-LUA-008: `zealot.*` Lua API bindings

## Context

Scripts need to query and mutate Zealot data. This task injects the `zealot` global table into the Lua VM with async functions backed by the service layer. All bindings use `mlua`'s `create_async_function` — they are Lua-callable async functions that yield into the Tokio runtime.

## Goal

Implement `crates/zealot-lua/src/bindings/` with all `zealot.*` functions.

## Requirements

### Module layout

```
crates/zealot-lua/src/bindings/
    mod.rs       — setup_zealot_globals(lua, context, services) -> Result
    items.rs     — zealot.items.* functions
    comments.rs  — zealot.comments.* functions
    utils.rs     — zealot.notify, zealot.log, zealot.now, zealot.date, zealot.event
```

### `bindings/mod.rs`

```rust
pub fn setup_zealot_globals(
    lua: &Lua,
    context: &RuleContext,
    services: Arc<ZealotServices>,
    account_id: Id,
    output_buf: Arc<Mutex<Vec<String>>>,
) -> mlua::Result<()> {
    let zealot = lua.create_table()?;

    items::register(&zealot, lua, services.clone(), account_id.clone())?;
    comments::register(&zealot, lua, services.clone(), account_id.clone())?;
    utils::register(&zealot, lua, context, account_id.clone(), output_buf)?;

    lua.globals().set("zealot", zealot)?;
    Ok(())
}
```

### `bindings/items.rs` — `zealot.items.*`

Each function below is registered as an async function on a `zealot.items` table.

**Helper: `item_to_lua(lua, item) → Table`**
Converts an `Item` into a Lua table with fields: `id`, `title`, `content`, `attributes` (table), `types` (array of strings), `links` (array of tables with `id` and `relationship`).

| Lua function | Backend call | Notes |
|---|---|---|
| `zealot.items.get(id)` | `ItemService::get_item_by_id` | Returns item table or nil |
| `zealot.items.get_by_title(title)` | `ItemService::get_item_by_title` | Returns item table or nil |
| `zealot.items.find_by_type(type_name)` | `ItemService::get_all(Some(type_name))` | Returns array |
| `zealot.items.search(term)` | `ItemService::search` | Returns array |
| `zealot.items.filter(filters)` | `ItemService::filter` | `filters` is array of `{key, op, value}` tables; convert to `AttributeFilterDto` |
| `zealot.items.create(title, opts?)` | `ItemService::add_item` | `opts`: `content`, `types[]`; returns item table |
| `zealot.items.update(id, opts)` | `ItemService::update_item` | `opts`: `title`, `content`; returns item table |
| `zealot.items.set_attribute(id, key, value)` | `AttributeService::set_value` | Returns bool |
| `zealot.items.assign_type(id, type_name)` | `ItemService::assign_type` | Returns bool |
| `zealot.items.delete(id)` | `ItemService::delete_item` | Returns bool |

All functions take `account_id` from the captured closure — scripts cannot override it.

### `bindings/comments.rs` — `zealot.comments.*`

| Lua function | Backend call |
|---|---|
| `zealot.comments.add(item_id, content)` | `CommentService::add_comment` |

### `bindings/utils.rs` — utility globals

```rust
// zealot.notify(msg) and zealot.log(msg) — append to output buffer
let buf_clone = output_buf.clone();
let notify = lua.create_function(move |_, msg: String| {
    buf_clone.lock().unwrap().push(msg);
    Ok(())
})?;
zealot.set("notify", notify.clone())?;
zealot.set("log", notify)?;

// zealot.now — ISO-8601 datetime string
zealot.set("now", chrono::Utc::now().to_rfc3339())?;

// zealot.date — YYYY-MM-DD
zealot.set("date", chrono::Local::now().format("%Y-%m-%d").to_string())?;

// zealot.event — nil or table depending on context
match context {
    RuleContext::Event(event) => {
        let event_table = event_to_lua(lua, event)?;
        zealot.set("event", event_table)?;
    }
    _ => {
        zealot.set("event", mlua::Value::Nil)?;
    }
}
```

**`event_to_lua` helper** — converts a `ZealotEvent` into a Lua table with a `kind` string field plus event-specific fields (see architecture doc for field list).

## Error handling in bindings

Bindings that call services should map errors to `mlua::Error::RuntimeError(msg)` using:
```rust
.map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
```

This surfaces service errors as Lua runtime errors, catchable with `pcall`.

## Dependencies

- TASK-LUA-007 (sandbox — these functions are injected after sandboxing)
- TASK-LUA-006 (service layer)

## Files to create

- `crates/zealot-lua/src/bindings/mod.rs`
- `crates/zealot-lua/src/bindings/items.rs`
- `crates/zealot-lua/src/bindings/comments.rs`
- `crates/zealot-lua/src/bindings/utils.rs`

## Verification

```rust
#[tokio::test]
async fn test_notify_captures_output() {
    let lua = new_sandbox().unwrap();
    let buf = Arc::new(Mutex::new(vec![]));
    // inject minimal zealot globals with just notify
    setup_zealot_globals(&lua, &RuleContext::Manual, ..., buf.clone()).unwrap();
    execute_script(&lua, r#"zealot.notify("hello")"#).await.unwrap();
    assert_eq!(buf.lock().unwrap().as_slice(), &["hello"]);
}
```

```bash
cargo test -p zealot-lua
```

# GAP-005 — Clarify item ID type in Lua rules documentation

## Problem

`docs/rules-engine.md` describes the item table structure as:

```
item.id    -- string, the item's unique ID
```

And the event section says:

```
zealot.event.item_id   -- string ID of the deleted item
zealot.event.item_id   -- string ID of the item the comment was added to
```

But the Lua bindings (`crates/zealot-lua/src/bindings/items.rs`) serialize item IDs as
`i64` integers:

```rust
t.set("id", i64::from(item.item_id))?;
```

And the bindings accept `i64` as input:

```rust
lua.create_async_function(move |lua, id: i64| { ... })
```

So `item.id` is a **Lua number (integer)**, not a string. Calling `zealot.items.get("abc123")`
would fail at the type boundary. Existing examples in the cookbook correctly pass integer
IDs, but the type annotation in the item table reference section is wrong.

## Fix

In `docs/rules-engine.md`, update the item table structure section:

```
item.id    -- integer (i64), the item's unique numeric ID
```

And in the `zealot.event` section:

```
zealot.event.item_id   -- integer, the ID of the deleted item
zealot.event.item_id   -- integer, the ID of the item the comment was added to
```

Also update the `zealot.items.get(id)` example if it uses a string literal.

## Files to change

- `docs/rules-engine.md` — item table structure section and `zealot.event` section

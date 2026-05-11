# TASK-LUA-010: Emit events from service mutations

## Context

The event system exists (TASK-LUA-003) but nothing calls `event_port.emit()` yet. This task wires event emission into `ItemService` and `CommentService` at every mutation point.

## Goal

Add `event_port: Arc<dyn EventPort>` to `ItemService` and `CommentService`, call `emit` after successful writes.

## Requirements

### `ItemService` changes

Add `event_port: Arc<dyn EventPort>` field. Update `ItemService::new` to accept it.

Emit after each successful mutation:

| Method | Event emitted |
|---|---|
| `add_item` | `ZealotEvent::ItemCreated { account_id, item }` |
| `update_item` | `ZealotEvent::ItemUpdated { account_id, item }` |
| `delete_item` | `ZealotEvent::ItemDeleted { account_id, item_id }` |
| `assign_type` | `ZealotEvent::TypeAssigned { account_id, item, type_name }` |
| `unassign_type` | `ZealotEvent::TypeUnassigned { account_id, item, type_name }` |
| `set_attribute` | `ZealotEvent::AttributeSet { account_id, item, attribute_key: key }` |

Pattern for each:
```rust
// After successful DB write, before returning Ok:
self.event_port.emit(ZealotEvent::ItemCreated {
    account_id: account.account_id.clone(),
    item: created_item.clone(),
});
return Ok(created_item);
```

### `CommentService` changes

Same pattern — add `event_port` field, emit `ZealotEvent::CommentAdded` from `add_comment`.

### Pass event_port through `ZealotServices::new`

`ZealotServices::new(ports, repos)` already has access to `ports.events`. Pass it when constructing `ItemService` and `CommentService`.

### Check for attribute service

`set_attribute` may live in `AttributeService` rather than `ItemService` depending on current code. Check and add emission there if so.

## Dependencies

- TASK-LUA-003 (event system)
- TASK-LUA-006 (services already set up)

## Files to modify

- `crates/zealot-app/src/services/item.rs`
- `crates/zealot-app/src/services/comment.rs`
- `crates/zealot-app/src/services/mod.rs` (update service construction)

## Verification

Integration test: create an item via `ItemService`, verify that a `ZealotEvent::ItemCreated` was emitted on the broadcast channel.

```bash
cargo check -p zealot-app
cargo test -p zealot-app
```

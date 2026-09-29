# GAP-015 — No guard on deleting an item type that items are using

## Problem

`crates/zealot-app/src/services/item_type.rs` `delete_item_type()` checks whether the
type is a system type (protected), but does **not** check whether any items are currently
assigned that type.

The repo runs a plain `DELETE FROM item_type WHERE type_id = $1`. The `item_item_type_link`
table has an `ON DELETE CASCADE` constraint, so all type assignments are silently removed
from every item using that type.

What makes this especially ironic: `GET /item_type/summary` already computes `item_count`
per type — so the query exists, it's just not used as a guard at deletion time.

## Impact

- All items assigned to (e.g.) "Task" silently lose that type.
- Filters by type, required-attribute enforcement, and any Lua rules using `find_by_type`
  break without any error.
- No warning, no confirmation prompt, no 409.

## Proposed fix

Re-use the existing summary count query before deletion:

```sql
SELECT COUNT(*) FROM item_item_type_link WHERE type_id = $1
```

If count > 0, return `409 Conflict`:
```
"Item type 'Task' is assigned to 37 items. Unassign first or pass force=true."
```

With `force=true`: proceed, letting the cascade handle the unassignment. This matches
the documented behaviour (items keep their data, lose the type label).

## Files to change

- `crates/zealot-app/src/services/item_type.rs` — add usage check before delete
- `crates/zealot-api/src/http/item_type.rs` — pass `force` param through
- `docs/http-api.md` — document 409 and `force` param

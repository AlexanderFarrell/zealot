# GAP-016 — Child items are orphaned silently when a parent is deleted

## Problem

`crates/zealot-app/src/services/item.rs` `delete_item()` (~line 346) deletes the item
and relies on the DB's `ON DELETE CASCADE` to remove `item_item_link` rows. This cleans
up link *records* — but the child items themselves still exist.

Result: if item "Website Redesign" (which has three child tasks) is deleted:
- The three tasks still exist in the database.
- Their parent links are gone (cascade removed).
- They now appear as root-level items with no parent.
- They disappear from their expected location in the nav tree and show up as orphaned
  top-level items — confusing and cluttering the UI.

There is no warning, no confirmation prompt, and no option to cascade-delete children.

## Impact

- Users who delete a project intending to clean up all related tasks find the tasks
  still floating at the top level.
- There is no "delete with children" option.
- This is one of the most common user-facing surprises in any hierarchical system.

## Proposed fix — Option A (recommended): Return 409 if children exist

Before deleting, check for children:

```sql
SELECT COUNT(*) FROM item_item_link
WHERE other_item_id = $1 AND relationship = 'parent' AND account_id = $2
```

If > 0, return `409 Conflict`:
```
"Item has 3 child items. Re-parent or delete them first, or pass cascade=true."
```

With `cascade=true`: recursively delete all descendants (depth-first, respecting the
same cascade=true behaviour on each child).

## Proposed fix — Option B: Document and warn in UI

Add a count of children to the item detail response (or as a header in the 200 DELETE
response), and add a confirmation dialog in the UI showing how many children will be
orphaned. This is less code but leaves the orphaning behaviour intact.

## Files to change

- `crates/zealot-app/src/services/item.rs` — children check before delete
- `crates/zealot-api/src/http/item.rs` — `cascade` query param
- `docs/http-api.md` — document behaviour

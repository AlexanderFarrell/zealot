# GAP-014 — No guard on deleting an attribute kind that items are using

## Problem

`crates/zealot-app/src/services/attribute.rs` `delete_attribute_kind()` calls the repo
directly with no usage check. The repo runs a plain `DELETE FROM attribute_kind WHERE key = $1`.

When an attribute kind is deleted while items have values for it:
- The `attribute` rows (per-item values) are **not** cascade-deleted — they remain in the
  database pointing at a kind that no longer exists.
- The UI can no longer render type information for those attributes.
- New items cannot use the key.
- The HTTP response returns 200 with no warning.

This is a silent data-integrity hazard. A user who cleans up "old" attribute kinds without
realising items still use them will corrupt their data silently.

## Steps to reproduce

1. Create attribute kind `Status` with dropdown values.
2. Create several items with `Status` set.
3. `DELETE /attribute/key/Status` — returns 200.
4. Items still have `attributes["Status"]` in the DB but no attribute kind definition exists.

## Proposed fix

Before deleting, count items that have a value for this key:

```sql
SELECT COUNT(*) FROM attribute WHERE key = $1 AND account_id = $2
```

If count > 0, return `409 Conflict` with a body like:
```
"Attribute kind 'Status' is used by 14 items. Delete those values first or pass force=true."
```

Add an optional `?force=true` query parameter that skips the check and also deletes all
per-item attribute values for that key (via a preceding `DELETE FROM attribute WHERE key = $1`).

## Files to change

- `crates/zealot-app/src/services/attribute.rs` — add usage check before delete
- `crates/zealot-api/src/http/attribute.rs` — pass `force` param to service
- `docs/http-api.md` — document the 409 behaviour and `force` param

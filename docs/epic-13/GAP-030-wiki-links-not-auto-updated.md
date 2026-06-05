# GAP-030 — Wiki links are not maintained automatically on item save

## Problem

`POST /item/rebuild-links` is the only way to sync wiki link records. The `create_item()`
and `update_item()` service methods do not extract or update `[[wiki link]]` references
from the saved content.

A user who writes `[[New Task]]` in an item body and saves it gets no live link. The link
appears rendered in the editor (client-side), but the server-side `item_item_link` record
that enables navigation and backlink tracking is not created until someone manually calls
`POST /item/rebuild-links`.

## Impact

- Navigation via wiki links in the sidebar or related-items panel silently fails for
  any link added since the last manual rebuild.
- Backlink counts and reverse-link queries are stale.
- The "auto-link wiki references" cookbook example in `docs/rules-engine.md` correctly
  describes using `zealot.items.get_by_title(ref)` to look up references — but this
  only works if the link records exist, which they won't for newly created items.
- Every new Zealot user who starts writing wiki links discovers this on their own.

## Proposed fix

After `create_item()` and `update_item()` succeed, call a new
`rebuild_links_for_item(item_id, account)` service method that:
1. Extracts all `[[Title]]` patterns from the saved content.
2. Looks up each title to find the referenced item's ID.
3. Upserts the corresponding `item_item_link` rows.
4. Removes any stale link rows for links that were in the old content but not the new.

The existing `rebuild_links_for_account()` can be refactored to call the per-item
version in a loop.

## Files to change

- `crates/zealot-app/src/services/item.rs` — add `rebuild_links_for_item`, call from save
- `crates/zealot-infra/src/repos/postgres/item_postgres.rs` — targeted link upsert/delete
- `docs/http-api.md` — note that `rebuild-links` is now mainly for one-off historical repair

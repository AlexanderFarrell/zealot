# Types Screens + Type Management Expansion

## Summary
Implement TASK-028 and TASK-029 together, and pull the minimum full `settings/types` management scope into this work so “Create type” is real now instead of pointing at a future placeholder.

This work will deliver:
- `/types` as a summary list of item types with assigned-attribute and item counts
- `/types/:title` as the editable type detail screen
- `/settings/types` as a type-management screen for add/edit-entry/delete
- backend/API support for type summaries, lookup by name, and deletion

## Public API / Interface Changes
- Add `GET /api/item_type/summary` returning `ItemTypeSummaryDto` with:
  `type_id`, `name`, `is_system`, `required_attributes_count`, `item_count`
- Add `GET /api/item_type/name/:name` for route-driven type lookup by title/name
- Add `DELETE /api/item_type/:id`
- Extend the item-type domain/API layer with a summary type plus `ItemTypeAPI.get_by_name()` and `ItemTypeAPI.get_summaries()`
- Extend navigator type navigation to support replace-mode for rename flow:
  `openType(title, mode?: 'push' | 'replace')`
- Extend `ItemTableView` with a configurable row-open handler so type detail can open `/item/:title` without changing existing screens that open by item id

## Implementation Changes
- Backend item-type stack:
  add repo/service/http methods for `summary`, `get by name`, and `delete`
- SQLite summary query:
  compute counts with `COUNT(DISTINCT ...)` on item links and attribute-kind links so joined rows do not overcount
- Delete behavior:
  only delete account-owned types; rely on existing FK cascade for item-type links and attribute-kind links; keep system types read-only/non-deletable
- `/types` screen:
  load summaries, render a simple table/list with name, assigned attributes count, and item count; row/name click opens type detail; “Create type” navigates to `/settings/types`
- `/settings/types`:
  make `SettingsScreen` a section dispatcher; for `types`, mount a dedicated type-management view; other sections stay placeholder
- Type-management view:
  show summary rows, inline add form with name only, create via `{ name, description: '', required_attributes: [] }`, then open the new `/types/:title` detail screen; each existing row gets `Edit` and `Delete`; `Edit` links to `/types/:title`
- `/types/:title` screen:
  load type by name, show back link, inline-editable name, and on successful rename update caches then replace the route with `/types/:newName`
- Assigned attribute kinds block:
  resolve `required_attributes` against the attribute-kind catalog, show name + base type, use a dropdown of unassigned kinds, add with `POST /attr_kind`, remove per row with `DELETE /attr_kind`
- Items block:
  load `GET /api/item?type=<type_name>` and render `ItemTableView` with columns `title` plus one column per currently assigned attribute kind; no create row; row click opens `/item/:title`
- Cache invalidation:
  mark type list/summary caches dirty after create, rename, assign, unassign, and delete so `/types`, `/settings/types`, and add-item type pickers stay fresh

## Test Plan
- Backend:
  verify summary counts are correct and distinct, `get by name` returns 404 when missing, delete removes custom types and cascades links, system types cannot be mutated/deleted
- UI/manual:
  verify `/types` counts and navigation, create from `/settings/types`, rename with URL replacement, add/remove attribute kinds, items table updates when assignments change, delete removes a custom type from settings
- Verification commands:
  run Rust checks/tests for the touched crates; run workspace TypeScript/build checks once the malformed [packages/core/package.json](/home/alexander/Projects/zealot/packages/core/package.json) is fixed, because npm currently fails before typecheck/build starts

## Assumptions / Defaults
- The expanded scope includes effective TASK-031 type CRUD now, but only for `settings/types`; other settings sections remain out of scope
- Attribute assignment uses attribute-kind keys, not numeric ids, because that is the current backend contract
- `ItemTableView` is the intended TASK-013-era replacement for the missing `ItemListView`
- System item types stay visible but are treated as read-only in detail/settings UI

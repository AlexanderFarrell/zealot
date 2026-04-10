# TASK-013: Item Table View + Child Inline Creation

## Summary
- Build a reusable `<item-table-view>` and wire it into the item screen for both `Children` and `Related`.
- Add inline item creation only to the `Children` table for now.
- The inline create row should capture `Title` plus any other visible editable cells before submit, then create the new item already linked as a child of the current item.

## Key Changes
- Extract the typed value-control logic from [packages/ui/src/views/attribute_editor.ts](/home/alexander/Projects/zealot/packages/ui/src/views/attribute_editor.ts) into a shared helper used by both the attribute editor and the table, so display/edit behavior stays consistent for `text`, `integer`, `decimal`, `date`, `week`, `dropdown`, `boolean`, `list`, and `item` fields.
- Add [packages/ui/src/views/item_table_view.ts](/home/alexander/Projects/zealot/packages/ui/src/views/item_table_view.ts) with exported config/types from [packages/ui/src/index.ts](/home/alexander/Projects/zealot/packages/ui/src/index.ts):
  - `ItemTableColumn` for `title`, `types`, and attribute-backed columns
  - `ItemTableViewConfig` with `items`, `columns`, `emptyMessage`, and optional `createRow`
  - `createRow` should include enabled state, submit label, parent/context item id, and success callback
- Table behavior:
  - sortable headers with `asc -> desc -> unsorted`
  - row navigation only when the click target is non-interactive
  - inline editing for title, types, and attributes using existing APIs
  - empty-state row with correct `colspan`
- Create-row behavior:
  - render a pinned add row above item rows when `createRow` is enabled
  - reuse the same editable cell controls as normal rows
  - require `Title`; treat other visible cells as optional unless a later usage opts into stricter validation
  - on submit, build `AddItemDto` from row values:
    - `title`
    - `content: ''`
    - `attributes` from non-empty attribute cells
    - `types` from the type cell
    - `links: [{ other_item_id: currentItemId, relationship: 'parent' }]` for the child table
  - on success, insert the returned item into table state, clear the create row, refocus title, and refresh the item-screen collections if needed
  - on failure, show an inline error message in the table chrome without clearing user input
- Update [packages/ui/src/screens/item_screen.ts](/home/alexander/Projects/zealot/packages/ui/src/screens/item_screen.ts):
  - replace the custom related renderer with shared `Children` and `Related` table sections
  - use default columns `Title`, `Type`, `Status`, `Priority`
  - enable the create row only for `Children`
  - keep loading/error/empty states separate per section
  - keep the existing collections toggle, but have it hide/show the combined collections wrapper

## Public Interfaces
- New exported UI surface:
  - `ItemTableView`
  - `ItemTableColumn`
  - `ItemTableViewConfig`
  - create-row config type for opt-in inline creation
- No backend endpoint changes. Creation continues through `POST /api/item/`, editing through existing item/type/attribute APIs.

## Test Plan
- Static validation:
  - `./node_modules/.bin/tsc -p packages/ui/tsconfig.json --noEmit`
  - `./node_modules/.bin/tsc -p apps/web/tsconfig.json --noEmit`
- Manual checks:
  - `Children` and `Related` both render as tables with independent loading/error/empty states
  - sort works for title, type, status, and priority
  - row clicks navigate, but inputs/selects/chips do not trigger navigation
  - editing title, type, status, and priority persists
  - child create row accepts title plus visible cell values, creates the item, and shows it immediately in `Children`
  - created child is linked to the current item and still appears after reload
  - create validation/error states stay inline and preserve entered values

## Assumptions
- `Related` remains read-only in this task.
- If inline creation is later enabled for `Related`, the default should be creating the new item with an outgoing `other` link to the current item.
- Workspace-wide `npm run typecheck --workspaces` is still blocked by the empty [packages/core/package.json](/home/alexander/Projects/zealot/packages/core/package.json), so direct `tsc` commands remain the acceptance gate.

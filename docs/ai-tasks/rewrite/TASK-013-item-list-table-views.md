# TASK-013: Item List & Table View Components

## Context
The original client had reusable list and table components for displaying collections of items. These are used by search results, children panels, related items panels, type views, and planner screens.

Run `git diff master -- client/src/features/item/item_list_view.ts` and `git diff master -- client/src/features/item/item_table.ts` to see the originals.

## Goal
Build reusable list and table view components for item collections.

## List View (`ItemListView`)
- Accepts an array of `Item` objects as input
- Each row: item title (link to `/item/:title`), type badge(s), optional secondary info
- Supports selection (single/multi) for bulk actions
- Empty state message when list is empty
- Optionally shows a loading skeleton while parent is fetching

## Table View (`ItemTableView`)
- Tabular display: columns for title, type, attributes (configurable)
- Column headers are sortable (client-side sort for now)
- Each row is clickable — navigates to item
- Empty state row

## Usage Sites
These components will be embedded in:
- Item screen children/related panels (TASK-010)
- Types screen (TASK-028, TASK-029)
- Analysis screen (TASK-036)
- Settings screens

## Files Likely Involved
- `packages/ui/src/` (new `item_list_view.ts`, `item_table_view.ts`)
- `packages/domain/src/item.ts`

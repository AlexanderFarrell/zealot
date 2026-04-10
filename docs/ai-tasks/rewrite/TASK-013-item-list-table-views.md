# TASK-013: Item List & Table View Components

## Context
The original client had reusable list and table components for displaying collections of items. These are used by search results, children panels, related items panels, type views, and planner screens.

Run `git diff master -- client/src/features/item/item_list_view.ts` and `git diff master -- client/src/features/item/item_table.ts` to see the originals.

We want to only bring table views into this version. List view will be re-envisioned later.

## Goal
Build reusable list and table view components for item collections.

## Table View (`ItemTableView`)
- Tabular display: columns for title, type, attributes (configurable)
- Column headers are sortable (client-side sort for now)
- Each row is clickable — navigates to item
- Empty state row
- Cells are editable. There were massive changes to individual attribute_item_views so make sure these changes such as item type reflect. 

For the item view, we likely will want to display at least title, type, and priority and status. 

Item view should display related items, not just children.

## Usage Sites
These components will be embedded in:
- Item screen children/related panels (TASK-010)
- Types screen (TASK-028, TASK-029)
- Analysis screen (TASK-036)
- Settings screens

## Files Likely Involved
- `packages/ui/src/` (new `item_list_view.ts`, `item_table_view.ts`)
- `packages/domain/src/item.ts`

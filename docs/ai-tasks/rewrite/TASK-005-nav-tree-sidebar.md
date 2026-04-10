# TASK-005: Navigation Tree Sidebar View

## Context
The sidebar in the original client has a navigation tree view that shows the item hierarchy. The new client's `side_bar.ts` is a stub.

Run `git diff master -- client/src/app/shell/sidebars/nav_view.ts` to see the original implementation.

## Goal
Build the hierarchical item tree sidebar that lets users navigate the item graph.

## Requirements
- Load root items on mount via `GET /api/item/` (no parent)
- Each item row is clickable — navigates to `/item/:title`
- Items with children show an expand/collapse chevron
- Expanding a row loads children via `GET /api/item/children/:id`
- Currently active item is highlighted
- Refresh when items are created or deleted (subscribe to the event system)

## Files Likely Involved
- `packages/ui/src/side_bar.ts` (or a new `nav_view.ts` alongside it)
- `packages/api/src/item.ts`
- `packages/engine/src/events.ts`

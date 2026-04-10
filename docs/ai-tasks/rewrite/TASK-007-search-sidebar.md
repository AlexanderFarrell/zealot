# TASK-007: Search Sidebar View

## Context
The original client had a search panel in the sidebar for quick item lookup.

Run `git diff master -- client/src/app/shell/sidebars/search_view.ts` to see the original.

## Goal
Build a search sidebar panel that queries items by term and lists results.

## Requirements
- Text input with debounced search (fire after ~300ms of no typing)
- Calls `GET /api/item/search?term=` with the current input value
- Results shown as a list — each row shows item title (and type badge if available)
- Clicking a result navigates to `/item/:title`
- Show empty state when no results found
- Show loading state while fetching

## Files Likely Involved
- `packages/ui/src/side_bar.ts` (or a new `search_view.ts`)
- `packages/api/src/item.ts`

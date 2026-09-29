# TASK-015: Inline Item Search Component

## Context
The original client had a typeahead/autocomplete component for selecting items by title. It is used in the "Add Item" modal (parent selector) and potentially in ZealotScript for inline item links.

Run `git diff master -- client/src/features/item/item_search_inline.ts` and `git diff master -- client/src/shared/generic_search.ts` to see the originals.

## Goal
Build a reusable inline item search / typeahead input component.

## Requirements
- Text input; as the user types, debounce and call `GET /api/item/search?term=`
- Show a dropdown list of matching items below the input
- Each result row shows item title (and type badge if available)
- Selecting a result fires a callback/event with the chosen `Item`
- Dismiss dropdown on Escape or blur
- Show "No results" when search returns empty
- Accessible: keyboard navigable (arrow keys, enter to select)

## Also: Global Search Modal
Run `git diff master -- client/src/features/search/search_modal.ts` to see the original global search modal. Build a full-screen modal variant of this component triggered by a hotkey (TASK-003).

## Files Likely Involved
- `packages/ui/src/` (new `item_search_inline.ts`, `search_modal.ts`)
- `packages/api/src/item.ts`

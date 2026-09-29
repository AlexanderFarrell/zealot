# TASK-012 + TASK-015: Add Item Modal and Inline/Global Item Search

## Summary
- Add a reusable inline item typeahead control and use it as the shared search primitive for parent selection and the existing item ID picker/chips inputs.
- Add light-DOM `AddItemModal` and `ItemSearchModal` overlays with single-instance `show()` helpers.
- Keep the desktop sidebar search tool; move `Ctrl/Cmd+O` to the new global search modal, and keep add-item entry points to the sidebar, mobile header, and `Ctrl/Cmd+N` only.

## Implementation Changes
- `packages/ui/src/views/item_search_inline.ts`
  - New custom element with a text input, 300ms debounce, stale-response guard, dropdown results, type badges, `No results`, keyboard nav (`ArrowUp/Down`, `Enter`, `Escape`), and blur dismissal.
  - Expose `focus()` and `clear()` helpers.
  - Fire both `OnSelect?: (item: Item) => void` and an `item-selected` event with `{ item }`.
  - When `Escape` closes an open dropdown, stop propagation so enclosing modals do not also close.

- `packages/ui/src/views/item_picker_input.ts` and `packages/ui/src/views/item_chips_input.ts`
  - Replace duplicated search/dropdown logic with `ItemSearchInline`.
  - Keep current chip UX and existing `OnChange` contracts; selection comes from the shared inline search component.

- `packages/ui/src/common/add_item_modal.ts`
  - New light-DOM modal appended to `document.body`, reusing `.modal_background` and `.inner_window`.
  - Title input autofocus; optional parent selector via `ItemSearchInline`; optional type `<select>` populated from `ItemTypeAPI.get_all()` and sorted by name.
  - Submit `ItemAPI.Add({ title, content: '', types?: [typeName], links?: [{ other_item_id: parentId, relationship: 'parent' }] })`.
  - Local validation: empty title shows an inline error without sending a request.
  - API validation: if `error.response` exists, read `response.text()` and show that inline; otherwise fall back to `error.message`.
  - On success, emit `Events.emit(ItemEvents.created, createdItem)`, close the modal, and navigate with `getNavigator().openItem(createdItem.Title)`.
  - Dismiss on backdrop click or `Escape`; if the parent-search dropdown is open, first `Escape` only closes that dropdown.

- `packages/ui/src/common/item_search_modal.ts`
  - New full-screen search modal wrapping `ItemSearchInline`.
  - Autofocus on open; selecting a result navigates with `getNavigator().openItemById(item.ItemID)` and then closes.
  - Add a single-instance `show()` helper so repeated hotkeys focus the existing modal instead of stacking overlays.

- Command wiring
  - Add engine-exported modal command constants:
    - `ModalCommands.newItem`
    - `ModalCommands.openGlobalSearch`
  - Export them from `packages/engine/src/index.ts`.
  - In `packages/engine/src/ui/tool_host.ts`, keep `ToolCommands.searchItems` as the sidebar-search action but remove its `Ctrl/Cmd+O` hotkey.
  - In `apps/web/src/web_client.ts`, register:
    - `Ctrl/Cmd+N` -> `AddItemModal.show()`
    - `Ctrl/Cmd+O` -> `ItemSearchModal.show()`
  - Update `packages/ui/src/shell/side_buttons.ts` so `New Item` runs the new modal command instead of the placeholder warning.
  - Update `packages/ui/src/shell/mobile_title_bar.ts` so:
    - add button opens `New Item`
    - search button opens `Open Global Search`
  - Leave the existing desktop sidebar search tool intact and reachable from the sidebar search button only.

- Exports
  - Export `ItemSearchInline`, `AddItemModal`, and `ItemSearchModal` from `packages/ui/src/index.ts`.

## Public Interfaces
- New UI surface:
  - `ItemSearchInline` with `focus()`, `clear()`, `OnSelect`, and `item-selected` event detail `{ item: Item }`
  - `AddItemModal.show()`
  - `ItemSearchModal.show()`
- New engine constants:
  - `ModalCommands.newItem`
  - `ModalCommands.openGlobalSearch`
- No backend API or DTO changes; reuse existing `POST /api/item/`, `GET /api/item/search`, and `GET /api/item_type/`.

## Test Plan
- Static checks:
  - `./node_modules/.bin/tsc -p packages/ui/tsconfig.json --noEmit`
  - `./node_modules/.bin/tsc -p apps/web/tsconfig.json --noEmit`
- Manual checks:
  - Sidebar `New Item`, mobile add button, and `Ctrl/Cmd+N` all open exactly one add-item modal.
  - Title-only create works; create with parent works; create with type works; blank title and backend validation errors render inline.
  - Successful create closes the modal, navigates to `/item/:title`, and refreshes the nav tree without a page reload.
  - Sidebar search button still opens the left search tool.
  - Mobile search button and `Ctrl/Cmd+O` open exactly one global search modal.
  - Inline search supports keyboard navigation, `Escape` closes the dropdown, blur dismisses results, and selection works from the add-item modal plus existing picker/chips inputs.

## Assumptions
- Do not add a separate item-screen launcher for the add-item modal; current inline child creation remains the on-item creation flow.
- Do not add a new desktop header-bar add button; desktop sidebar plus mobile header plus hotkey are enough.
- Keep API error-body parsing local to the new add-item modal instead of broadening global fetch error handling in this task.

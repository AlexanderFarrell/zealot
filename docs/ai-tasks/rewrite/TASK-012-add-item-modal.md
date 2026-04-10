# TASK-012: Add Item Modal

## Context
The original client had a modal for creating new items, accessible from the sidebar and the item screen.

Run `git diff master -- client/src/features/item/add_item_modal.ts` and `git diff master -- client/src/features/item/add_item_scope.ts` to see the original.

## Goal
Build the "Add Item" modal dialog.

## Requirements
- Trigger: button in the header bar and/or sidebar, plus a keyboard shortcut (TASK-003)
- Fields:
  - Title (required text input, auto-focused)
  - Parent item (optional — uses inline item search, TASK-015)
  - Item type (optional — dropdown populated from `GET /api/item/type`)
- Submit calls `POST /api/item/` with `{ title, parent_id?, type? }`
- On success: close modal, navigate to the new item (`/item/:title`), emit an event to refresh the nav tree
- On error: show inline validation message
- Dismiss with Escape key or clicking backdrop

## Dependencies
- TASK-001 (Router) — navigate after creation
- TASK-015 (Inline Item Search) — parent selector

## Files Likely Involved
- `packages/ui/src/` (new `add_item_modal.ts`)
- `packages/api/src/item.ts`
- `packages/api/src/item_type.ts`

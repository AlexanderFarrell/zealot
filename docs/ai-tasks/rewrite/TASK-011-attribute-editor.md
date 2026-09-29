# TASK-011: Attribute Editor Component

## Context
Items have dynamic key/value attributes with typed "attribute kinds" (text, integer, decimal, date, week, dropdown, boolean, list). The original client had a shared attribute editing component used in both the item screen and the sidebar.

Run `git diff master -- client/src/features/item/view/attributes_view.ts` and `git diff master -- client/src/features/item/view/attribute_item_view.ts` to see the original.

## Goal
Build a reusable attribute editor component that works in both the item screen (TASK-010) and attributes sidebar (TASK-008).

## Requirements
- Renders a list of attribute key/value pairs
- Each row has:
  - Key label (click-to-rename → calls `PATCH /api/item/:id/attr/rename`)
  - Value field — input type depends on attribute kind:
    - `text` → text input
    - `integer` / `decimal` → number input
    - `date` → date picker
    - `week` → week picker
    - `dropdown` → select with configured options
    - `boolean` → checkbox/toggle
    - `list` → chips input (reuse `chips_input.ts`)
  - Delete button → calls `DELETE /api/item/:id/attr/:key` (with confirmation)
- "Add attribute" row at the bottom — type a key name and press enter to create
- All saves call `PATCH /api/item/:id/attr` with `{ [key]: value }`
- Attribute kinds are loaded once from `GET /api/item/kind` and cached

## Files Likely Involved
- `packages/ui/src/` (new `attribute_editor.ts` or similar)
- `packages/api/src/attribute.ts`
- `packages/api/src/attribute_kind.ts`
- `packages/ui/src/chips_input.ts`

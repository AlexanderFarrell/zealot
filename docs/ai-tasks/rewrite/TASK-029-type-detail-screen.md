# TASK-029: Type Detail Screen

## Context
The `/types/:title` route shows a specific item type: its assigned attribute kinds and the items of that type.

Run `git diff master -- client/src/features/types/type_screen.ts` to see the original.

## Goal
Build the type detail screen at `/types/:title`.

## Requirements

### Header
- Type name (editable inline)
- Back link to `/types`

### Assigned Attribute Kinds
- List of attribute kinds assigned to this type
- Each row: attribute kind name, type (text/integer/etc.)
- "Add attribute kind" — opens a selector dropdown populated from `GET /api/attribute/`
  - Calls `POST /api/item_type/:id/attr_kind` with the kind IDs to assign
- Remove button on each row → `DELETE /api/item_type/:id/attr_kind`

### Items of This Type
- Load items of this type via `GET /api/item?type=<type_name>`
- Display using the ItemListView component (TASK-013)
- Clicking an item navigates to `/item/:title`

## Dependencies
- TASK-001 (Router)
- TASK-013 (ItemListView)
- TASK-028 (Types List Screen)

## Files Likely Involved
- `packages/ui/src/` (new `type_screen.ts`)
- `packages/api/src/item_type.ts`
- `packages/api/src/item.ts`

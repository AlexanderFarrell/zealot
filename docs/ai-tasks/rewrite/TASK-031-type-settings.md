# TASK-031: Item Type Settings Screen

## Context
`/settings/types` lets admins create and manage item types and their attribute assignments.

Run `git diff master -- client/src/features/settings/type_settings_screen.ts` to see the original.

## Goal
Build the item type settings screen.

## Requirements

### Listing
- Load all item types via `GET /api/item_type/`
- Show each in a list: type name, number of assigned attribute kinds

### Add Type
- "Add" button → name input → `POST /api/item_type/` on submit

### Edit
- Each type row expands or links to an edit view
- Edit name: `PATCH /api/item_type/:id`
- Assign/unassign attribute kinds (same UI as TASK-029)

### Delete
- Delete button with confirmation
- Calls `DELETE /api/item_type/:id`

## Dependencies
- TASK-001 (Router)
- TASK-030 (Attribute Kind Settings — attribute kinds must exist to assign)

## Files Likely Involved
- `packages/ui/src/` (new `type_settings_screen.ts`)
- `packages/api/src/item_type.ts`

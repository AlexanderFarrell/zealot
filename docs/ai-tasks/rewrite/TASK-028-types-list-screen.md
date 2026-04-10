# TASK-028: Types List Screen

## Context
The `/types` route shows all defined item types and allows navigation to a type's detail view.

Run `git diff master -- client/src/features/types/types_screen.ts` to see the original.

## Goal
Build the types list screen at `/types`.

## Requirements
- Load all item types via `GET /api/item/type`
- Show each type as a card or row: type name, number of assigned attributes, count of items with that type
- Clicking a type navigates to `/types/:title`
- Link to create a new type (opens a modal or navigates to type settings)

## Dependencies
- TASK-001 (Router)
- TASK-029 (Type Detail Screen)

## Files Likely Involved
- `packages/ui/src/` (new `types_screen.ts`)
- `packages/api/src/item_type.ts`

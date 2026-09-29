# TASK-014: Drag & Drop for Item Hierarchy

## Context
The original client supported dragging items onto other items to reassign the parent relationship. This was used in the navigation tree sidebar.

Run `git diff master -- client/src/features/item/drag_helper.ts` to see the original implementation.

## Goal
Implement drag-and-drop for item hierarchy reordering/reparenting.

## Requirements
- Items in the nav tree sidebar (TASK-005) can be dragged
- Dropping an item onto another item sets the dragged item's parent to the drop target
- Calls `PATCH /api/item/:id` with `{ parent_id: <drop_target_id> }`
- Visual affordances: dragging cursor, drop target highlight, drag ghost
- Dropping on empty space (root) removes the parent (sets `parent_id: null`)

## Implementation Notes
- Use the native HTML5 Drag and Drop API — no external library needed
- Keep the drag logic isolated in a helper/mixin so it can be applied to any item row

## Dependencies
- TASK-005 (Nav Tree Sidebar)

## Files Likely Involved
- `packages/engine/src/` (new `drag_helper.ts`)
- `packages/ui/src/` nav tree component

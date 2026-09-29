# TASK-008: Item Attributes Sidebar View

## Context
The original sidebar had a panel showing the attributes of the currently viewed item, allowing quick inline editing without opening the full item screen.

Run `git diff master -- client/src/app/shell/sidebars/item_attributes_view.ts` to see the original.

## Goal
Build the attributes sidebar panel that shows and edits the selected item's attributes.

## Requirements
- Displays when an item is currently loaded/selected
- Shows all attributes as key/value pairs
- Each value is inline-editable (click to edit, blur/enter to save)
- Calls `PATCH /api/item/:id/attr` on save
- Supports all attribute kinds: text, integer, decimal, date, week, dropdown, boolean, list
- Add attribute button — shows an input for key name, creates via the same patch endpoint
- Delete attribute via the delete button on each row

## Dependencies
- TASK-010 (Item Screen) for the "currently selected item" state
- TASK-011 (Attribute Editor) — may share components

## Files Likely Involved
- `packages/ui/src/side_bar.ts` (or a new `item_attributes_view.ts`)
- `packages/api/src/attribute.ts`

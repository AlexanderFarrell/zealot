# TASK-030: Attribute Kind Settings Screen

## Context
`/settings/attributes` lets admins manage the base attribute kinds (the types of values that attributes can hold).

Run `git diff master -- client/src/features/settings/attr_settings_screen.ts` to see the original.

## Goal
Build the attribute kind settings screen.

## Requirements

### Listing
- Load all attribute kinds via `GET /api/attribute/`
- Show each in a table: name, base type (text, integer, decimal, date, week, dropdown, boolean, list)

### Add Attribute Kind
- "Add" button opens a form: name input, base type selector
- Calls `POST /api/attribute/` on submit
- Appends the new kind to the list

### Edit
- Each row has an edit action — inline or modal form
- For `dropdown` kinds: show a field to manage the allowed option values (comma-separated or chip input)
- Saves via `PATCH /api/attribute/id/:id`

### Delete
- Delete button with confirmation
- Calls `DELETE /api/attribute/id/:id`

## Dependencies
- TASK-001 (Router — settings route)

## Files Likely Involved
- `packages/ui/src/` (new `attr_kind_settings_screen.ts`)
- `packages/api/src/attribute_kind.ts`

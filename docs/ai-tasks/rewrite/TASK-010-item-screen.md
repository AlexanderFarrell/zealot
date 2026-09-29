# TASK-010: Item Screen

## Context
The item screen is the primary view of the app — it displays and edits a single item (wiki-style page). It is reached via `/item/:title` and `/item_id/:id`. The original implementation is substantial.

Run `git diff master -- client/src/features/item/item_screen.ts` and `git diff master -- client/src/features/item/view/` to see the original.

## Goal
Build the full item viewer/editor screen.

## Requirements

### Loading
- Accept a `title` or `item_id` param from the router
- Fetch item via `GET /api/item/title/:title` or `GET /api/item/id/:id`
- Show a loading state while fetching; show a "not found" state if the item doesn't exist

### Header
- Display item title (editable inline — saves on blur via `PATCH /api/item/:id`)
- Show assigned type badges
- Action buttons: copy link, open in new tab, delete item (with confirmation)

### Content Area
- Render the item's content using the ZealotScript editor (TASK-016–020)
- Content is editable; auto-save or explicit save button

### Attributes Panel
- Show all item attributes as key/value pairs (shares logic with TASK-008 and TASK-011)
- Inline add/edit/delete

### Related & Children Panels
- List related items (`GET /api/item/related/:id`) with navigation links
- List child items (`GET /api/item/children/:id`) with navigation links
- Both panels allow adding/removing relationships

### Analysis Summary (optional for MVP)
- Small inline analysis widget showing recent activity for this item

## Dependencies
- TASK-001 (Router) — route params
- TASK-016–020 (ZealotScript Editor) — content editing
- TASK-011 (Attribute Editor) — attributes panel

## Files Likely Involved
- `packages/ui/src/` (new `item_screen.ts`)
- `packages/api/src/item.ts`
- `packages/api/src/attribute.ts`

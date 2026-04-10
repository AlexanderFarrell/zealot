# TASK-025: Comments View Component

## Context
Comments appear in two places in the original client: on item screens (threaded discussion) and on planner day views (journal entries).

Run `git diff master -- client/src/features/comments/comments_view.ts` and `git diff master -- client/src/features/comments/comment_shared.ts` to see the originals.

## Goal
Build a reusable comments view component.

## Requirements

### Display
- Renders a list of comments with: author, timestamp, content text
- Empty state when there are no comments

### Add Comment
- Text area at the bottom with a "Post" button
- Calls `POST /api/comments` with `{ item_id?, date?, content }`
- Clears input on success, appends the new comment to the list

### Edit Comment
- Each comment has an "Edit" button (own comments only)
- Inline edit — replaces comment text with a textarea pre-filled with current content
- Save calls `PATCH /api/comments/:id`

### Delete Comment
- Each comment has a "Delete" button (own comments only)
- Confirmation before deleting
- Calls `DELETE /api/comments/:id`, removes from list on success

## Usage Sites
- Daily planner screen (TASK-021) — `date` context
- Item screen (TASK-010) — `item_id` context

## Files Likely Involved
- `packages/ui/src/` (new `comments_view.ts`)
- `packages/api/src/comment.ts`

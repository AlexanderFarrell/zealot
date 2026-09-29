# TASK-021: Daily Planner Screen

## Context
The daily planner shows all items, repeat statuses, and comments for a given calendar day. It is the most-used planner view.

Run `git diff master -- client/src/features/planner/day_screen.ts` to see the original.

## Goal
Build the daily planner screen at `/planner/daily` and `/planner/daily/:date`.

## Requirements

### Data Loading
- Date param is ISO format `yyyy-MM-dd`; default to today if absent
- Load in parallel:
  - `GET /api/planner/day/:date` — items scheduled for this day
  - `GET /api/repeat/day/:date` — repeat/habit statuses for this day
  - `GET /api/comments/day/:date` — journal/log comments for this day

### Layout
- Header: date display with previous/next day navigation arrows, "Today" shortcut
- Items section: list of items for the day (use TASK-013 ItemListView)
- Repeat section: list of habit/repeat items each with a status toggle (done / skipped / none) and optional comment
  - Status change calls `PUT /api/repeat/status` with `{ item_id, date, status, comment }`
- Comments section: journal entries for the day (use TASK-025 CommentsView)

### Navigation
- Previous/next arrows update the URL (pushes history)
- Clicking an item navigates to `/item/:title`

## Dependencies
- TASK-001 (Router)
- TASK-013 (ItemListView)
- TASK-025 (CommentsView)

## Files Likely Involved
- `packages/ui/src/` (new `daily_planner_screen.ts`)
- `packages/api/src/planner.ts` (check if it exists, create if not)
- `packages/api/src/repeat.ts`
- `packages/api/src/comment.ts`

# TASK-022: Weekly Planner Screen

## Context
The weekly planner shows a 7-column view of a full week, one column per day.

Run `git diff master -- client/src/features/planner/week_screen.ts` to see the original.

## Goal
Build the weekly planner screen at `/planner/weekly` and `/planner/weekly/:date`.

## Requirements

### Data Loading
- Date param is ISO week format; default to current week if absent
- Load `GET /api/planner/week/:date` — items grouped by day for the week

### Layout
- Header: week range label (e.g. "Apr 7 – Apr 13, 2026"), previous/next week arrows
- 7 columns (Mon–Sun), each showing:
  - Day label and date number
  - List of items for that day
  - Today's column is visually highlighted
- Clicking a day header navigates to the daily planner for that day
- Clicking an item navigates to `/item/:title`

### Navigation
- Previous/next arrows update the URL

## Dependencies
- TASK-001 (Router)
- TASK-021 (Daily Planner) — shares some day-column logic

## Files Likely Involved
- `packages/ui/src/` (new `weekly_planner_screen.ts`)
- `packages/api/src/planner.ts`

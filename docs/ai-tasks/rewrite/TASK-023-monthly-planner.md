# TASK-023: Monthly Planner Screen

## Context
The monthly planner shows a calendar grid for an entire month.

Run `git diff master -- client/src/features/planner/month_screen.ts` to see the original.

## Goal
Build the monthly planner screen at `/planner/monthly` and `/planner/monthly/:date`.

## Requirements

### Data Loading
- Date param is `yyyy-MM` format; default to current month
- Load `GET /api/planner/month/:month/year/:year`

### Layout
- Header: month + year label, previous/next month arrows
- Calendar grid: 7-column (Mon–Sun) weekly rows
- Each day cell shows:
  - Day number
  - Items scheduled that day (truncated with overflow indicator if many)
- Today's cell is highlighted
- Days outside the current month are dimmed

### Interactions
- Clicking a day cell navigates to the daily planner for that date
- Clicking an item navigates to `/item/:title`

## Dependencies
- TASK-001 (Router)

## Files Likely Involved
- `packages/ui/src/` (new `monthly_planner_screen.ts`)
- `packages/api/src/planner.ts`

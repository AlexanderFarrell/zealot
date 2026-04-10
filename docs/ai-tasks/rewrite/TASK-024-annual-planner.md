# TASK-024: Annual Planner Screen

## Context
The annual planner provides a year-at-a-glance view.

Run `git diff master -- client/src/features/planner/year_screen.ts` to see the original.

## Goal
Build the annual planner screen at `/planner/annual` and `/planner/annual/:year`.

## Requirements

### Data Loading
- Year param is a 4-digit year; default to current year
- Load `GET /api/planner/year/:year`

### Layout
- Header: year label, previous/next year arrows
- 12 mini monthly calendar grids arranged in a grid (3×4 or 4×3)
- Each month grid shows day cells with an indicator if items exist that day
- Today is highlighted

### Interactions
- Clicking a day navigates to the daily planner for that date
- Clicking a month header navigates to the monthly planner for that month

## Dependencies
- TASK-001 (Router)
- TASK-023 (Monthly Planner) — shares mini-calendar logic

## Files Likely Involved
- `packages/ui/src/` (new `annual_planner_screen.ts`)
- `packages/api/src/planner.ts`

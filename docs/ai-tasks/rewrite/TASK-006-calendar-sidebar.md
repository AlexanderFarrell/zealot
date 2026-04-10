# TASK-006: Calendar Widget Sidebar

## Context
The original client's sidebar included a mini calendar widget for quick navigation to planner views.

Run `git diff master -- client/src/app/shell/sidebars/calendar_view.ts` to see the original implementation.

## Goal
Build a mini monthly calendar widget for the sidebar.

## Requirements
- Shows current month by default with previous/next month navigation arrows
- Each day is clickable — navigates to `/planner/daily/:date` (ISO format `yyyy-MM-dd`)
- Today's date is visually highlighted
- Days that have planner items may be subtly marked (optional, can skip for MVP)

## Files Likely Involved
- `packages/ui/src/side_bar.ts` (or a new `calendar_view.ts` alongside it)
- Router (TASK-001) for navigation

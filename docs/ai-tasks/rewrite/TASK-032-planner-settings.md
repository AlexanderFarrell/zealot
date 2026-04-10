# TASK-032: Planner Settings Screen

## Context
`/settings/planner` configures planner behavior such as which item types/attributes appear in planner views.

Run `git diff master -- client/src/features/settings/planner_settings_screen.ts` to see the original.

## Goal
Build the planner settings screen.

## Requirements
- Run the git diff above to enumerate the exact settings available
- Typical settings include: default planner view (daily/weekly/monthly), which item types show in the planner, attribute used for scheduling date
- Changes persist via TASK-002 (Settings Persistence) or direct API calls

## Dependencies
- TASK-001 (Router)
- TASK-002 (Settings Persistence)

## Files Likely Involved
- `packages/ui/src/` (new `planner_settings_screen.ts`)

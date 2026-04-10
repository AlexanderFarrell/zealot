# TASK-034: Data Settings Screen

## Context
`/settings/data` allows importing, exporting, and backing up application data.

Run `git diff master -- client/src/features/settings/data_settings_screen.ts` to see the original.

## Goal
Build the data settings screen.

## Requirements
- Run the git diff above to enumerate the exact operations supported
- Typical features: export all data as JSON/ZIP, import from a backup, wipe/reset data (dangerous, requires confirmation)
- Each action should show a progress indicator and success/error notification (TASK-004)

## Dependencies
- TASK-001 (Router)
- TASK-004 (Notifications)

## Files Likely Involved
- `packages/ui/src/` (new `data_settings_screen.ts`)

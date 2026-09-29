# TASK-033: Wiki Settings Screen

## Context
`/settings/wiki` configures wiki/documentation behavior.

Run `git diff master -- client/src/features/settings/wiki_settings_screen.ts` to see the full list of settings and their UI.

## Goal
Build the wiki settings screen.

## Requirements
- Run the git diff above to enumerate the exact settings
- Typical settings: default home item, navigation behavior, display preferences
- Changes persist via TASK-002 (Settings Persistence)

## Dependencies
- TASK-001 (Router)
- TASK-002 (Settings Persistence)

## Files Likely Involved
- `packages/ui/src/` (new `wiki_settings_screen.ts`)

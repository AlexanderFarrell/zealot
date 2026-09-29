# TASK-002: Settings Persistence

## Context
The original client (`master:client/`) persists client-side settings locally and syncs them to the server via `PATCH /api/account/settings`. Settings govern things like sidebar state, planner defaults, and UI preferences.

Run `git diff master -- client/src/shared/settings.ts` to see the original settings implementation.

## Goal
Implement client-side settings storage that persists to localStorage and syncs to the server.

## Requirements
- Store settings in localStorage (key/value) so they survive page reload
- On login, load settings from the server (`GET /api/account/details`) and merge with local
- On settings change, debounce and sync to server (`PATCH /api/account/settings`)
- Expose a typed settings object accessible across all UI components

## Settings to Support
Run `git diff master -- client/src/shared/settings.ts` and `git diff master -- client/src/features/settings/` to enumerate all setting keys.

## Files Likely Involved
- `packages/engine/src/settings.ts` (may already exist as a stub)
- `packages/api/src/auth.ts` (for the sync endpoint)

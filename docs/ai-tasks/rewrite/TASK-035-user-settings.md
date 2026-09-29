# TASK-035: User Settings Screen

## Context
`/settings/user` lets the user manage their account: display name, password change, preferences.

Run `git diff master -- client/src/features/settings/user_settings_screen.ts` to see the original.

## Goal
Build the user settings screen.

## Requirements
- Load current user details via `GET /api/account/details`
- Display name: editable text field, saved via `PATCH /api/account/settings`
- Password change: current password + new password + confirm, submitted to the appropriate endpoint (check backend for the correct route)
- Theme / UI preferences (if any — check original)
- Logout button → calls `GET /api/account/logout`, then redirects to login

## Dependencies
- TASK-001 (Router)

## Files Likely Involved
- `packages/ui/src/` (new `user_settings_screen.ts`)
- `packages/api/src/auth.ts`

# TASK-004: Toast Notifications, Loading States & Error Handling

## Context
`packages/engine/src/popups.ts` exists but isn't wired to any UI actions. The app needs user feedback for async operations — successes, errors, and in-progress states.

Run `git diff master -- client/src/shared/popups.ts` to see the original popups implementation.

## Goal
Wire notifications and add consistent loading/error handling across the app.

## Requirements

### Toast Notifications
- Success toasts for: item saved, item deleted, comment added, file uploaded, settings saved
- Error toasts for: API failures, validation errors
- Use the existing `popups.ts` infrastructure — don't replace it

### Loading States
- Add a loading indicator (spinner or skeleton) to screens while data is fetching
- Buttons that trigger async actions should show a disabled/loading state during the request

### Error Boundaries
- Catch unhandled errors in screen components and show a friendly error UI rather than a blank screen
- Log errors to the console with enough context to debug

### Confirmation Dialogs
- Destructive actions (delete item, delete file, delete attribute) must require confirmation before proceeding
- Simple modal with "Cancel" / "Confirm" buttons

## Files Likely Involved
- `packages/engine/src/popups.ts`
- `packages/ui/src/` (new shared modal/dialog component)

# TASK-003: Wire Hotkey System to UI

## Context
`packages/engine/src/hotkeys.ts` and `packages/engine/src/commands.ts` exist but no default hotkeys are registered and no UI listens for them. The original client had keyboard shortcuts wired to navigation and common actions.

Run `git diff master -- client/src/shared/hotkeys.ts` and `git diff master -- client/src/shared/command_runner.ts` to see the original implementations.

## Goal
Register default hotkeys and wire the command system so keyboard shortcuts trigger UI actions.

## Hotkeys to Implement (from original)
- Open global search modal
- Navigate home (`/`)
- Open daily planner
- Focus sidebar search
- Any other hotkeys from the original — run the git diff to find them all

## Implementation Notes
- Hotkeys should be registered at app startup in `apps/web/src/core.ts` or similar bootstrap file
- Should respect focus state (e.g. don't trigger navigation shortcuts when typing in an input)
- The command system should be the bridge — hotkeys dispatch commands, commands trigger router/UI

## Files Likely Involved
- `packages/engine/src/hotkeys.ts`
- `packages/engine/src/commands.ts`
- `apps/web/src/core.ts`

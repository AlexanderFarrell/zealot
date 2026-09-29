# TASK-LUA-FE-004: Add Rules to sidebar navigation

## Context

The rules screen exists but is not accessible via the UI. This task wires `RulesScreen` into the router and adds a navigation entry to the sidebar, following the exact pattern used for other screens (e.g. Analysis, Media).

## Goal

Make `/rules` a navigable route and add a sidebar button for it.

## Requirements

### Router (`apps/web/src/web_navigator.ts` or equivalent)

Add a `/rules` route that renders `<rules-screen>`:
- Follow the pattern of other routes (e.g. `/analysis`, `/media`)
- Import `RulesScreen` from `packages/ui`
- Ensure `customElements.define('rules-screen', RulesScreen)` is called (likely already in the screen file's module)

### Navigator interface (`packages/engine/src/ui/navigator.ts` or similar)

Add `openRules(): void` to the `Navigator` interface. Implement it in `WebNavigator` as `this.navigate('/rules')`.

### Sidebar button

Find where other sidebar navigation buttons are defined (check `apps/web/src/web_tool_host.ts` or the sidebar component). Add a "Rules" button:
- Icon: use the existing rules/automation SVG icon from `packages/content/src/`. If none exists, use a generic "settings" or "code" icon and note that a dedicated icon should be added later.
- Tooltip: `"Rules & Automation"`
- Calls `navigator.openRules()`

### Update TASK-037

TASK-037 (`rules_screen.ts`) can now be considered superseded by TASK-LUA-FE-003. Add a note to TASK-037 pointing to the LUA tasks.

## Dependencies

- TASK-LUA-FE-003 (screen must exist)
- TASK-001 (Router)

## Files to modify

- `apps/web/src/web_navigator.ts` (or router file)
- The Navigator interface file
- The sidebar/shell component file
- `packages/ui/src/index.ts` (export `RulesScreen` if not already)

## Verification

1. Reload the app
2. Click the Rules sidebar button
3. Verify the URL changes to `/rules` and `<rules-screen>` renders
4. Verify navigating away and back preserves history (browser back button works)

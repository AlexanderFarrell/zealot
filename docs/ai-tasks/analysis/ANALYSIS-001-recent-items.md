# ANALYSIS-001: Recent Items Tab

## Context
Users need a quick way to jump back to recently created items without remembering a title or navigating a tree. The `ItemID` field is a monotonically increasing integer and serves as a reliable creation-order proxy until a proper `updated_at` field is added to the backend.

## Goal
Add a **Recent** tab to the analysis tab bar that shows the last 20–30 items by creation order.

## Requirements

### Data
- Fetch all items via `itemApi.GetAll()` and sort client-side by `ItemID` descending, taking the top 30.
- No new API endpoint needed.

### UI
- Add **Recent** as a new tab entry in `renderTabBar()` alongside Analysis / Specify / Working.
- Display results in `ItemTableView` with columns: Title, Type, Status, Date.
- Show "No recent items." if the result is empty.

### Navigation
- Add `analysis_recent` to the `AppLocation` union in `packages/engine/src/ui/navigator.ts`.
- Add `openAnalysisRecent()` to the `Navigator` interface.
- Register a hotkey: `Ctrl+Shift+R`.
- Wire the new route in `apps/web/src/`.

## Files
- `packages/ui/src/screens/analysis_screen.ts` — new `RecentScreen` class, tab entry in `renderTabBar()`
- `packages/engine/src/ui/navigator.ts` — `analysis_recent` location, `openAnalysisRecent()` method, hotkey registration
- `apps/web/src/` — route wiring

## Verification
- `tsc --noEmit` in `packages/ui` and `packages/engine` — no errors.
- Navigate to `/analysis/recent` — table renders with items sorted newest-first.
- Tab bar shows **Recent** highlighted.
- Hotkey `Ctrl+Shift+R` navigates to the recent tab from anywhere.
- Empty state renders without crash.

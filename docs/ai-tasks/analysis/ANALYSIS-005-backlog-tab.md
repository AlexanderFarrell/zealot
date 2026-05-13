# ANALYSIS-005: Backlog Tab

## Context
Items without a `Date` attribute are invisible to every planner view (daily/weekly/monthly/annual all filter by date). There is no existing way to see them in aggregate. This tab surfaces the undated backlog so users can review, prioritize, and schedule items.

## Goal
Add a **Backlog** tab showing all items that have no `Date` attribute set.

## Requirements

### Data
- Fetch all items via `itemApi.GetAll()` and filter client-side: `items.filter(i => !i.Attributes?.['Date'])`.
- Alternatively, if the backend `Filter` endpoint supports a "key absent" query, prefer that — but check first; the client-side filter is the safe fallback.

### UI
- New tab **Backlog** in `renderTabBar()`.
- Show item count in the tab label: **Backlog (14)**. Count is determined after data loads; update the tab label once loaded.
- Display in `ItemTableView` with columns: Title, Type, Priority, AP (no Date column — these items don't have one).
- Empty state: "No items in Backlog."

### Navigation
- Add `analysis_backlog` to the `AppLocation` union in `packages/engine/src/ui/navigator.ts`.
- Add `openAnalysisBacklog()` to the `Navigator` interface.
- Wire the new route in `apps/web/src/`.

## Files
- `packages/ui/src/screens/analysis_screen.ts` — new `BacklogScreen` class, tab entry in `renderTabBar()`
- `packages/engine/src/ui/navigator.ts` — `analysis_backlog` location, `openAnalysisBacklog()`
- `apps/web/src/` — route wiring

## Verification
- `tsc --noEmit` in `packages/ui` and `packages/engine` — no errors.
- Navigate to the Backlog tab — only undated items appear.
- Tab label shows correct count after load.
- Creating a new item without a date and refreshing shows it in the backlog.
- Empty state renders without crash.

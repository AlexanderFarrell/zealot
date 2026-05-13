# ANALYSIS-006: Overdue Items Tab

## Context
Items with a `Date` in the past and a non-terminal status (not Complete or Cancelled) represent silent failures — commitments that were never acted on or closed. No existing view surfaces these; they just quietly accumulate. This tab makes them visible so users can reschedule or close them.

## Goal
Add an **Overdue** tab showing items whose `Date` is before today and whose `Status` is neither Complete nor Cancelled.

## Requirements

### Data
- Fetch items with `Date < today` using `itemApi.Filter([{ key: 'Date', op: 'lt', value: today, list_mode: 'any' }])`.
- Filter client-side to exclude terminal statuses: `status !== 'Complete' && status !== 'Cancelled'`.
- `today` = `DateTime.now().toISODate()`.

### UI
- New tab **Overdue** in `renderTabBar()`.
- Show item count in the tab label: **Overdue (7)**. Update after load.
- Display in `ItemTableView` with columns: Title, Type, Status, Date, Priority.
- Empty state: "No overdue items." (this is a success state — style it positively if desired).

### Navigation
- Add `analysis_overdue` to the `AppLocation` union in `packages/engine/src/ui/navigator.ts`.
- Add `openAnalysisOverdue()` to the `Navigator` interface.
- Wire the new route in `apps/web/src/`.

## Files
- `packages/ui/src/screens/analysis_screen.ts` — new `OverdueScreen` class, tab entry in `renderTabBar()`
- `packages/engine/src/ui/navigator.ts` — `analysis_overdue` location, `openAnalysisOverdue()`
- `apps/web/src/` — route wiring

## Verification
- `tsc --noEmit` in `packages/ui` and `packages/engine` — no errors.
- Navigate to the Overdue tab — only past-dated, non-terminal items appear.
- Complete items with past dates do not appear.
- Tab label shows correct count after load.
- Empty state ("No overdue items.") renders without crash.

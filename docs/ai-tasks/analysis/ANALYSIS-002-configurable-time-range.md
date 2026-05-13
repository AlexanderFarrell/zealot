# ANALYSIS-002: Configurable Time Range on Analysis Tab

## Context
The analysis tab hardcodes 30 days in `AnalysisUtils.getItemsByDays(30)`. Users who want a week-level pulse or a quarterly view have no way to adjust it.

## Goal
Add a time-range selector above the charts on the main Analysis tab so the user can switch between 7, 30, 90, and 365 days.

## Requirements

### UI
- Render a `<select>` element above the charts with options: 7 days, 30 days, 90 days, 365 days.
- Default to 30 days.
- On change: re-fetch data and re-render all charts and graphs in place (clear and rebuild the chart section, not the whole screen).
- Update the subheading to match: "Last 7 Days", "Last 30 Days", etc.

### Persistence
- Persist the selected value in `localStorage` under the key `zealot_analysis_days`.
- Read and apply it on initial render.

### Implementation Notes
- The existing `AnalysisUtils.getItemsByDays(days)` already accepts a `days` parameter — just thread the selection through.
- Extract the chart-building portion of `AnalysisScreen.render()` into a helper (e.g. `renderCharts(items, days, container)`) so it can be called on change without rebuilding the tab bar and heading.

## Files
- `packages/ui/src/screens/analysis_screen.ts` — `AnalysisScreen.render()`, new `renderCharts()` helper

## Verification
- `tsc --noEmit` in `packages/ui` — no errors.
- Switching the dropdown re-renders charts with correctly scoped data.
- Selection is preserved across page reloads.
- Subheading matches the selected range.

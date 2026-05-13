# ANALYSIS-003: Type Distribution Pie Chart

## Context
The current analysis pie row shows Status, Priority, and AP distributions. The app has a rich type system (`Item.Types: ItemTypeRef[]`) but it is entirely absent from the dashboard. Adding a type chart makes the system's categorization visible at a glance.

## Goal
Add a **Types** pie chart to the analysis pie row on the main Analysis tab.

## Requirements

### Data
- Group items by their primary type name: `item.Types[0]?.Name ?? 'Untyped'`.
- Reuse the existing `AnalysisUtils.groupBySum` pattern — but since `Types` is an array, not a flat attribute, write a small dedicated helper `groupByType(items)` that returns `Record<string, number>`.

### UI
- Add the new `PieChartView` to `renderPieRow()` as a 4th chart: caption **"Types"**.
- No layout changes needed — `analysis-charts-row` flexbox will accommodate a 4th chart.

## Files
- `packages/ui/src/screens/analysis_screen.ts` — new `groupByType()` helper in `AnalysisUtils`, extend `renderPieRow()`

## Verification
- `tsc --noEmit` in `packages/ui` — no errors.
- Types pie renders with correct labels and counts.
- Items with no types show under "Untyped".

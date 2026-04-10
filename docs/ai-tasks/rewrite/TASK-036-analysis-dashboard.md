# TASK-036: Analysis Dashboard Screen

## Context
The `/analysis` route showed a 30-day analytics dashboard with line graphs and pie charts summarizing item activity.

Run `git diff master -- client/src/features/analysis/analysis_screen.ts` and `git diff master -- client/src/features/analysis/analysis.ts` to see the original. Also check `git diff master -- client/src/shared/graphs.ts` for the chart utilities.

## Goal
Build the analysis dashboard screen.

## Requirements

### Data
- Run the git diff above to see which API endpoints were used (likely item filter + repeat endpoints)
- Compute 30-day rolling stats client-side or fetch pre-aggregated data from the backend

### Charts
The original had:
- **Line graph**: score over time (last 30 days)
- **Line graph**: completed items over time (last 30 days)
- **Pie chart**: status distribution (e.g. done / in-progress / blocked)
- **Pie chart**: priority distribution
- **Pie chart**: action points distribution

### Implementation
- Wire to the existing `packages/engine/src/graphs.ts` utilities (`LineGraphView`, `PieChartView`)
- If the graph utilities are stubs, implement them using SVG or Canvas
- Show a loading skeleton while data is fetching

## Dependencies
- TASK-001 (Router)

## Files Likely Involved
- `packages/ui/src/` (new `analysis_screen.ts`)
- `packages/engine/src/graphs.ts`
- `packages/api/src/item.ts` (filter endpoint)

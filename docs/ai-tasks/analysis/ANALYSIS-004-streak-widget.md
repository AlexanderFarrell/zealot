# ANALYSIS-004: Streak / Productivity Streak Widget

## Context
The scorecard line graph shows raw daily score but provides no motivational summary. A "current streak" counter gives users a quick health signal at a glance and rewards consistent daily completion.

## Goal
Add a stat row above the line graphs showing current streak, best streak, and average daily score.

## Requirements

### Data (pure client-side, no new API calls)
Computed from the already-fetched items and the existing time-series helpers:

- **Current streak**: consecutive days (counting back from today) where at least 1 item was completed or cancelled.
- **Best streak**: the longest such run within the fetched window.
- **Avg score**: mean of non-zero score days within the window (rounded to nearest integer).

Write a `computeStreakStats(items, days)` helper in `AnalysisUtils` that returns `{ current: number; best: number; avgScore: number }`.

### UI
- Render a `<div class="analysis-stats-row">` containing three `<span>` stat chips before the line graphs:
  - `Streak: N days`
  - `Best: N days`
  - `Avg score: N`
- Keep it plain text with light styling — no extra libraries.

## Files
- `packages/ui/src/screens/analysis_screen.ts` — `computeStreakStats()` in `AnalysisUtils`, stat row in `AnalysisScreen.render()` (inside the `renderCharts` helper once ANALYSIS-002 is done, or directly in `render()` before that)

## Verification
- `tsc --noEmit` in `packages/ui` — no errors.
- Stats row renders with correct values when items have completions.
- Zero-day streak shows `Streak: 0 days` without errors.
- All-zero score window shows `Avg score: 0`.

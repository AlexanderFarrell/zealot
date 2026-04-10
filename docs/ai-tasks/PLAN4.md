# TASK-021 to TASK-024 Planner Implementation

## Summary
Implement the four planner screens with the repo’s current data model and your requested scope overrides:

- `TASK-021` daily planner stays day-based and shows:
  - a day-scoped item table
  - repeat status/comment controls
  - no comments section in this batch
- `TASK-022` to `TASK-024` use the legacy scoped planner model instead of the rewrite calendar layouts:
  - weekly screen = items filtered by `Week`
  - monthly screen = items filtered by `Month` + `Year`
  - annual screen = items filtered by `Year`
- drag-and-drop is omitted
- planner item collections use `ItemTableView`, not a list view
- planner tables support inline creation with period defaults

## Key Changes
### Backend and API
- Add a planner HTTP surface with four legacy endpoints returning flat `ItemDto[]` collections:
  - `GET /planner/day/:date`
  - `GET /planner/week/:week`
  - `GET /planner/month/:month/year/:year`
  - `GET /planner/year/:year`
- Implement a `PlannerService` on top of existing item filtering/hydration, not a new repo layer.
- Register the planner routes in the Rust HTTP router and expose a new TS `PlannerAPI` from the frontend API package.
- Keep method signatures DateTime-based on the TS side:
  - `GetForDay(date)`
  - `GetForWeek(date)`
  - `GetForMonth(date)`
  - `GetForYear(date)`

### UI Behavior
- Daily planner:
  - parse `yyyy-MM-dd`
  - header with previous day, next day, and Today
  - load day items and repeats in parallel
  - render items in `ItemTableView` with `Title`, `Type`, `Status`, `Priority`
  - render repeats grouped by `Time of Day` with immediate-save status control and comment field
  - omit the comments section entirely
- Weekly planner:
  - parse ISO week `kkkk-'W'WW`
  - header with previous week, next week, and This Week
  - render one scoped item table for the selected week, not 7 day columns
- Monthly planner:
  - parse `yyyy-MM`
  - header with previous month, next month, and This Month
  - render one scoped item table for the selected month/year, not a calendar grid
- Annual planner:
  - parse `yyyy`
  - header with previous year, next year, and This Year
  - render one scoped item table for the selected year, not mini month calendars
- In all planner tables:
  - row click navigates via item id (`openItemById`)
  - empty states are explicit
  - invalid route params show a friendly error state with a shortcut back to the current period

### Item Table Extension
- Extend `ItemTableView` create-row config to support default scoped attributes merged into submission.
- Use these defaults for inline creation:
  - daily: `Date=<selected day>`, `Status="To Do"`, `Priority=3`
  - weekly: `Week=<selected ISO week>`, `Status="To Do"`, `Priority=3`
  - monthly: `Month=<selected month>`, `Year=<selected year>`, `Status="To Do"`, `Priority=3`
  - annual: `Year=<selected year>`, `Status="To Do"`, `Priority=3`
- Visible planner table columns stay consistent across screens: `Title`, `Type`, `Status`, `Priority`.

## Public Interfaces and Types
- Add `PlannerAPI` to the TS API package and wire it into the root `ZealotAPI`.
- Add planner service registration in the Rust app service container.
- No new domain DTOs are required if all planner routes return `ItemDto[]`.
- Extend `ItemTableCreateRowConfig` with planner-safe default attributes so inline creation works without planner-specific table forks.

## Test Plan
- Automated:
  - workspace TS typecheck
  - web build
  - Rust compile/test pass for server crates
- Manual:
  - `/planner/daily`, `/planner/weekly`, `/planner/monthly`, `/planner/annual` default routes resolve to current period URLs
  - explicit valid params load the expected scoped items
  - previous/next/current buttons update URL and reload data
  - inline create on each planner table creates an item with the correct scoped attributes
  - daily repeat status/comment updates persist
  - item row clicks open the correct item
  - invalid date/week/month/year params do not crash and show fallback guidance

## Assumptions and Defaults
- Daily comments are intentionally skipped in this batch; `TASK-025` remains deferred.
- Weekly/monthly/annual follow the legacy scoped-table model, overriding the rewrite task calendar layouts.
- Planner navigation opens items by id, not by title slug.
- This plan targets the repo’s current SQLite-backed path; making unfinished Postgres attribute/filter repos planner-ready is out of scope for this batch.

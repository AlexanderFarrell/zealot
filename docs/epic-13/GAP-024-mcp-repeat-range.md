# GAP-024 — MCP repeat entries are single-day only; no week/range query

## Problem

`apps/mcp/src/tools/planner.rs` `get_repeat_entries` takes a single `date`
parameter. There is no way to retrieve habit completion data across a range of dates.

The HTTP API also only has `GET /repeat/day/{date}` (single day), so this is a gap
at both the HTTP and MCP layers (see also GAP-033 for the HTTP layer fix).

## Impact

An agent asked to "summarise my habit completion this week" must make 7 separate
`get_repeat_entries` calls and aggregate the results. This is slow (7 round-trips)
and wastes context window tokens on repeated boilerplate responses.

Weekly review rules and habit streak calculations are the most common agent workflows
that require multi-day repeat data.

## Proposed fix

This ticket tracks the MCP layer. It depends on GAP-033 adding `GET /repeat/range`
to the HTTP API first.

Once the HTTP endpoint exists, add:

```rust
#[rmcp::tool(description = "Get repeat/habit entries for a date range. Returns all habit completion statuses between start_date and end_date (inclusive). Format: YYYY-MM-DD.")]
pub async fn get_repeat_entries_for_range(&self, p: GetRepeatRangeInput) -> ...

pub struct GetRepeatRangeInput {
    pub start_date: String,  // YYYY-MM-DD
    pub end_date: String,    // YYYY-MM-DD
}
```

## Dependencies

- GAP-033 (HTTP repeat range endpoint) must be implemented first.

## Files to change

- `apps/mcp/src/tools/planner.rs` — add `get_repeat_entries_for_range`
- `docs/mcp.md` — update Repeats/habits tool table

# GAP-023 — MCP has no annual planner tool

## Problem

`apps/mcp/src/tools/planner.rs` exposes `get_day_plan`, `get_week_plan`, and
`get_month_plan`, but there is no `get_year_plan`. The HTTP API endpoint
`GET /planner/year/{year}` exists and works.

## Impact

Agents doing annual review prompts ("what did I have scheduled in 2026?") or
year-in-review summaries cannot use the planner. They must work around it by
making 12 monthly calls and aggregating, or by searching items with a `Year`
attribute directly.

## Proposed fix

Add a `get_year_plan` tool to `apps/mcp/src/tools/planner.rs`:

```rust
#[rmcp::tool(description = "Get all items scheduled for a given year. Returns items with a Date or Week attribute falling within that year.")]
pub async fn get_year_plan(&self, p: GetYearPlanInput) -> Result<CallToolResult, McpError>

pub struct GetYearPlanInput {
    /// The year (e.g. 2026)
    pub year: i64,
}
```

Calls `GET /planner/year/{year}`.

## Files to change

- `apps/mcp/src/tools/planner.rs` — add `get_year_plan` tool
- `docs/mcp.md` — add to Planner tools table, update tool count

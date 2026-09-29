# GAP-013 — Add `filter_items` tool to MCP server

## Problem

`docs/mcp.md` explicitly lists this as a known limitation:

> "No filtered queries. The HTTP API supports rich attribute-based filtering
> (`GET /item/filter`), but MCP does not expose this endpoint. Work around it with
> `search_items` (keyword search) or `list_items` with `type_filter`, then filter
> client-side."

This is a significant gap for agent workflows. The HTTP API's `POST /item/filter` endpoint
supports filtering by attribute value, operator, and list mode. Without it in MCP, agents
must:
1. Fetch all items of a type
2. Filter client-side in their context window

This is both slow and token-expensive for accounts with many items.

Common agent use cases that require filtering:
- "What are my incomplete high-priority tasks?" → filter `Status != Complete AND Priority = High`
- "Show goals with no due date" → filter `Date` not present
- "Which habits haven't been completed this week?" → filter repeat status

## Proposed implementation

Add a `filter_items` tool to `apps/mcp/src/tools/wiki.rs`:

```rust
#[derive(Debug, Deserialize)]
pub struct FilterItemsInput {
    /// Array of attribute filters. Each filter has: key, op (eq/ne/gt/lt/gte/lte/ilike), value.
    pub filters: Vec<AttributeFilter>,
}

#[derive(Debug, Deserialize)]
pub struct AttributeFilter {
    pub key: String,
    pub op: String,
    pub value: serde_json::Value,
    #[serde(default = "default_list_mode")]
    pub list_mode: String,
}
```

The tool calls `POST /item/filter` on the HTTP client and returns the matching items.

## Tool description for agents

> Filter items by attribute values. Pass an array of filters, each with `key` (attribute
> name), `op` (one of: eq, ne, gt, lt, gte, lte, ilike), and `value`. All filters are
> ANDed. Example: `[{"key": "Status", "op": "eq", "value": "In Progress"}]`.

## Files to change

- `apps/mcp/src/tools/wiki.rs` — add `filter_items` tool
- `docs/mcp.md` — update tool count, add entry to tool reference table, remove
  "No filtered queries" known limitation

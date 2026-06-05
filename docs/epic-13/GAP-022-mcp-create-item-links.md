# GAP-022 — MCP `create_item` cannot set links (parent, tag, topic)

## Problem

`apps/mcp/src/tools/wiki.rs` `create_item` tool only accepts `title`, `content`, and
`attributes`. The underlying HTTP API (`POST /item/`) fully supports a `links` array
for establishing parent, tag, topic, and other relationships at creation time.

## Impact

When an agent is asked to "create a task under the Website Redesign project", it must:
1. `create_item` — creates the item
2. `update_item` — sets the parent link

Two round-trips, and agents often forget the second step. In practice, items created
via MCP are frequently orphaned at the root with no parent relationship.

This is the single most common agent failure mode for hierarchical content creation.

## Proposed fix

Add an optional `links` parameter to the `create_item` tool:

```rust
pub links: Option<Vec<ItemLinkInput>>,

// where:
pub struct ItemLinkInput {
    pub other_item_id: i64,
    pub relationship: String, // "parent", "blocks", "tag", "topic", "other"
}
```

Pass it through to the HTTP API's existing `links` field.

**Tool description addition:**
> Optional `links`: array of `{other_item_id, relationship}` objects. Use `relationship: "parent"` to make this item a child of another item. Common relationships: `"parent"`, `"tag"`, `"topic"`, `"blocks"`.

## Files to change

- `apps/mcp/src/tools/wiki.rs` — add `links` to `CreateItemInput` and the HTTP call
- `docs/mcp.md` — update `create_item` tool reference

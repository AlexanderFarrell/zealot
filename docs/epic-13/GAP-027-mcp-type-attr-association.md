# GAP-027 — MCP cannot manage type-attribute kind associations

## Problem

`apps/mcp/src/tools/automation.rs` has tools to create and update item types and
attribute kinds, but no tools for:
- `POST /item_type/{type_id}/attr_kind` — associate attribute kinds with a type
- `DELETE /item_type/{type_id}/attr_kind` — disassociate attribute kinds

Without these, an agent that helps a user set up their initial Zealot schema (types
with required attributes) can create the types and the attribute kinds, but cannot
link them. The "required attributes" for each type remain empty.

## Impact

Setting up Zealot's schema via MCP is half-implemented. An agent following the
quickstart tutorial:
1. Creates `Task` type ✓
2. Creates `Status` attribute kind ✓
3. Creates `Date` attribute kind ✓
4. **Cannot** make `Status` and `Date` required for `Task` type ✗

Users who want to use `is_valid()` type checking or UI validation see no effect from
agent-assisted setup.

## Proposed fix

Add two tools to `apps/mcp/src/tools/automation.rs`:

```rust
#[rmcp::tool(description = "Associate attribute kinds with an item type. Pass an array of attribute keys to add as associated (and optionally required) attributes for this type.")]
pub async fn associate_attributes_with_type(&self, p: AssociateAttributesInput) -> ...

#[rmcp::tool(description = "Disassociate attribute kinds from an item type. Pass an array of attribute keys to remove.")]
pub async fn disassociate_attributes_from_type(&self, p: AssociateAttributesInput) -> ...

pub struct AssociateAttributesInput {
    pub type_id: i64,
    pub attribute_keys: Vec<String>,
}
```

## Files to change

- `apps/mcp/src/tools/automation.rs` — add the two tools
- `docs/mcp.md` — add to Schema tools table

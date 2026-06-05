# GAP-026 — MCP automation tool descriptions only list 4 of 10 trigger kinds

## Problem

`apps/mcp/src/tools/automation.rs` `create_rule` tool description shows trigger
examples for only 4 kinds:

```json
{"kind": "manual"}
{"kind": "cron", "expression": "0 9 * * *"}
{"kind": "on_item_create"}
{"kind": "on_type_assign", "type_name": "Goal"}
```

The server supports 10 trigger kinds total. The following are **not documented**
in the MCP tool description and agents have no way to discover them:
- `on_item_update`
- `on_item_delete`
- `on_comment_add`
- `on_type_unassign` (with optional `type_name`)
- `on_attribute_set` (with optional `attribute_key`)
- `interval` (with `seconds` field)

## Impact

Agents presented with "create a rule that fires when Status is set to Complete"
don't know `on_attribute_set` exists. They'll use `on_item_update` instead (less
targeted) or create a cron workaround. The most powerful trigger kinds go unused.

## Proposed fix

Update the `create_rule` and `update_rule` tool descriptions in `automation.rs` to
list all 10 trigger kinds with their fields and a brief description:

```
Trigger kinds:
  {"kind": "manual"}
  {"kind": "cron", "expression": "0 9 * * *"}           -- 5-field cron
  {"kind": "interval", "seconds": 3600}                  -- every N seconds
  {"kind": "on_item_create"}
  {"kind": "on_item_update"}
  {"kind": "on_item_delete"}
  {"kind": "on_comment_add"}
  {"kind": "on_type_assign", "type_name": "Task"}        -- type_name optional (null = any type)
  {"kind": "on_type_unassign", "type_name": "Task"}      -- type_name optional
  {"kind": "on_attribute_set", "attribute_key": "Status"} -- attribute_key optional (null = any key)
```

## Files to change

- `apps/mcp/src/tools/automation.rs` — update `create_rule` and `update_rule`
  tool description strings
- `docs/mcp.md` — update Trigger formats section with all 10 kinds

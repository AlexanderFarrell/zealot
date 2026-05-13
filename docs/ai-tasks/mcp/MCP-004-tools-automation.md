# MCP-004: Automation Tools

## Goal

Implement 15 MCP tools for Zealot's automation and schema management: rules (Lua scripts triggered by events), item types (categories), and attribute kinds (custom field definitions). All in `apps/mcp/src/tools/automation.rs`.

## Depends On

MCP-001, MCP-002

## Background

### Rules

Rules are Lua scripts that run in response to triggers (item creation, cron schedule, manual invocation, etc.). The `create_rule` and `update_rule` tools accept a `trigger` JSON object matching one of these shapes:

```json
{"kind": "manual"}
{"kind": "cron", "expression": "0 9 * * 1-5"}
{"kind": "interval", "seconds": 3600}
{"kind": "on_item_create"}
{"kind": "on_item_update"}
{"kind": "on_item_delete"}
{"kind": "on_comment_add"}
{"kind": "on_type_assign", "type_name": "Goal"}
{"kind": "on_attribute_set", "attribute_key": "due"}
```

The `script` field is a Lua string. See the `automate_workflow` prompt (MCP-006) for embedded Lua API documentation.

### Item Types

Item types are labels (like "Goal", "Project", "Habit") that categorize items. Each type can have required attributes and an optional icon/color.

### Attribute Kinds

Attribute kinds define the schema for custom attributes. Each has a key, a base type, and optional config:

- `text` — plain string
- `integer` / `decimal` — numeric
- `date` — date string
- `week` — ISO week string
- `boolean` — true/false
- `dropdown` — `config: {"values": ["option1", "option2"]}`
- `item` — reference to another item ID
- `list` — list of values

## File: `apps/mcp/src/tools/automation.rs`

```rust
use rmcp::{
    tool,
    model::{CallToolResult, Content},
    Error as McpError,
    handler::server::tool::Parameters,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::tools::ZealotServer;
use super::api_err;

// ── Rules ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RuleIdParam {
    pub rule_id: i64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateRuleParams {
    pub name: String,
    pub description: Option<String>,
    /// Trigger as a JSON object. Examples:
    /// {"kind":"manual"}, {"kind":"cron","expression":"0 9 * * *"},
    /// {"kind":"on_item_create"}, {"kind":"on_type_assign","type_name":"Goal"}
    pub trigger: serde_json::Value,
    /// Lua script body. Use zealot.* API to interact with Zealot.
    pub script: String,
    /// Whether the rule is active (default true)
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateRuleParams {
    pub rule_id: i64,
    pub name: Option<String>,
    pub description: Option<String>,
    pub trigger: Option<serde_json::Value>,
    pub script: Option<String>,
    pub enabled: Option<bool>,
}

// ── Item Types ────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ItemTypeNameParam {
    pub name: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ItemTypeIdParam {
    pub type_id: i64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateItemTypeParams {
    /// Unique name for the type (e.g. "Goal", "Project", "Habit")
    pub name: String,
    pub description: Option<String>,
    /// Icon identifier or emoji
    pub icon: Option<String>,
    /// Hex color string (e.g. "#4A90E2")
    pub color: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateItemTypeParams {
    pub type_id: i64,
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
}

// ── Attribute Kinds ───────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AttributeKeyParam {
    pub key: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AttributeIdParam {
    pub kind_id: i64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateAttributeKindParams {
    /// Unique key (slug) for the attribute (e.g. "due_date", "priority")
    pub key: String,
    pub description: Option<String>,
    /// Base type: text, integer, decimal, date, week, boolean, dropdown, item, list
    pub base_type: String,
    /// Type-specific config (e.g. {"values":["low","medium","high"]} for dropdown)
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateAttributeKindParams {
    pub kind_id: i64,
    pub description: Option<String>,
    pub config: Option<serde_json::Value>,
}

// ── Tool implementations ─────────────────────────────────────────────────────

impl ZealotServer {
    // Rules

    #[tool(description = "List all automation rules. Rules are Lua scripts triggered by events, schedules, or manually.")]
    pub async fn list_rules(&self) -> Result<CallToolResult, McpError> {
        let rules: serde_json::Value = self.client.get("/rule/").await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&rules).unwrap_or_default())]))
    }

    #[tool(description = "Get a single rule by its numeric ID, including the full Lua script.")]
    pub async fn get_rule(&self, Parameters(p): Parameters<RuleIdParam>) -> Result<CallToolResult, McpError> {
        let rule: serde_json::Value = self.client.get(&format!("/rule/{}", p.rule_id)).await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&rule).unwrap_or_default())]))
    }

    #[tool(description = "Create a new automation rule with a trigger and Lua script. The trigger is a JSON object (see description for shapes). The script uses the zealot.* Lua API.")]
    pub async fn create_rule(&self, Parameters(p): Parameters<CreateRuleParams>) -> Result<CallToolResult, McpError> {
        let body = json!({
            "name": p.name,
            "description": p.description,
            "trigger": p.trigger,
            "script": p.script,
            "enabled": p.enabled.unwrap_or(true),
        });
        let rule: serde_json::Value = self.client.post("/rule/", &body).await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&rule).unwrap_or_default())]))
    }

    #[tool(description = "Update an existing rule's name, description, trigger, script, or enabled state.")]
    pub async fn update_rule(&self, Parameters(p): Parameters<UpdateRuleParams>) -> Result<CallToolResult, McpError> {
        let body = json!({
            "name": p.name,
            "description": p.description,
            "trigger": p.trigger,
            "script": p.script,
            "enabled": p.enabled,
        });
        let rule: serde_json::Value = self.client.patch(&format!("/rule/{}", p.rule_id), &body).await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&rule).unwrap_or_default())]))
    }

    #[tool(description = "Delete a rule by ID.")]
    pub async fn delete_rule(&self, Parameters(p): Parameters<RuleIdParam>) -> Result<CallToolResult, McpError> {
        self.client.delete(&format!("/rule/{}", p.rule_id)).await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text("rule deleted".to_string())]))
    }

    #[tool(description = "Run a rule immediately, regardless of its trigger. Returns the execution result including any output or errors from the Lua script.")]
    pub async fn run_rule(&self, Parameters(p): Parameters<RuleIdParam>) -> Result<CallToolResult, McpError> {
        let result: serde_json::Value = self.client.post(&format!("/rule/{}/run", p.rule_id), &json!({})).await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&result).unwrap_or_default())]))
    }

    // Item Types

    #[tool(description = "List all item types defined in this Zealot instance (e.g. Goal, Project, Habit, Task).")]
    pub async fn list_item_types(&self) -> Result<CallToolResult, McpError> {
        let types: serde_json::Value = self.client.get("/item_type/").await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&types).unwrap_or_default())]))
    }

    #[tool(description = "Get a specific item type by its name. Returns the type definition including required attributes.")]
    pub async fn get_item_type_by_name(&self, Parameters(p): Parameters<ItemTypeNameParam>) -> Result<CallToolResult, McpError> {
        let t: serde_json::Value = self.client.get(&format!("/item_type/name/{}", urlencoding::encode(&p.name))).await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&t).unwrap_or_default())]))
    }

    #[tool(description = "Create a new item type with a name, optional description, icon, and color.")]
    pub async fn create_item_type(&self, Parameters(p): Parameters<CreateItemTypeParams>) -> Result<CallToolResult, McpError> {
        let body = json!({
            "name": p.name,
            "description": p.description,
            "icon": p.icon,
            "color": p.color,
        });
        let t: serde_json::Value = self.client.post("/item_type/", &body).await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&t).unwrap_or_default())]))
    }

    #[tool(description = "Update an existing item type's name, description, icon, or color.")]
    pub async fn update_item_type(&self, Parameters(p): Parameters<UpdateItemTypeParams>) -> Result<CallToolResult, McpError> {
        let body = json!({
            "name": p.name,
            "description": p.description,
            "icon": p.icon,
            "color": p.color,
        });
        let t: serde_json::Value = self.client.patch(&format!("/item_type/{}", p.type_id), &body).await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&t).unwrap_or_default())]))
    }

    #[tool(description = "Delete an item type by its numeric ID. Items with this type are not deleted.")]
    pub async fn delete_item_type(&self, Parameters(p): Parameters<ItemTypeIdParam>) -> Result<CallToolResult, McpError> {
        self.client.delete(&format!("/item_type/{}", p.type_id)).await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text("item type deleted".to_string())]))
    }

    // Attribute Kinds

    #[tool(description = "List all attribute kind definitions (the schema for custom item attributes).")]
    pub async fn list_attribute_kinds(&self) -> Result<CallToolResult, McpError> {
        let kinds: serde_json::Value = self.client.get("/attribute/").await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&kinds).unwrap_or_default())]))
    }

    #[tool(description = "Get a specific attribute kind by its key (slug).")]
    pub async fn get_attribute_by_key(&self, Parameters(p): Parameters<AttributeKeyParam>) -> Result<CallToolResult, McpError> {
        let kind: serde_json::Value = self.client.get(&format!("/attribute/key/{}", urlencoding::encode(&p.key))).await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&kind).unwrap_or_default())]))
    }

    #[tool(description = "Create a new attribute kind (custom field schema). base_type is one of: text, integer, decimal, date, week, boolean, dropdown, item, list. For dropdown, config should be {\"values\":[\"opt1\",\"opt2\"]}.")]
    pub async fn create_attribute_kind(&self, Parameters(p): Parameters<CreateAttributeKindParams>) -> Result<CallToolResult, McpError> {
        let body = json!({
            "key": p.key,
            "description": p.description,
            "base_type": p.base_type,
            "config": p.config.unwrap_or(json!({})),
        });
        let kind: serde_json::Value = self.client.post("/attribute/", &body).await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&kind).unwrap_or_default())]))
    }

    #[tool(description = "Update an attribute kind's description or config by its numeric ID.")]
    pub async fn update_attribute_kind(&self, Parameters(p): Parameters<UpdateAttributeKindParams>) -> Result<CallToolResult, McpError> {
        let body = json!({
            "description": p.description,
            "config": p.config,
        });
        let kind: serde_json::Value = self.client.patch(&format!("/attribute/id/{}", p.kind_id), &body).await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(serde_json::to_string_pretty(&kind).unwrap_or_default())]))
    }
}
```

## Verify

```bash
cargo check -p zealot-mcp

# List rules:
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"list_rules","arguments":{}}}' | \
  ZEALOT_API_KEY=mykey cargo run -p zealot-mcp 2>/dev/null | jq '.result'
```

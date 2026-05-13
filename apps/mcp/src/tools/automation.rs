use rmcp::{
    model::{CallToolResult, Content},
    ErrorData as McpError,
    handler::server::wrapper::Parameters,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::tools::{ZealotServer, api_err};

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
    /// Lua script body — use the zealot.* API to interact with Zealot
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

fn pretty(v: serde_json::Value) -> String {
    serde_json::to_string_pretty(&v).unwrap_or_default()
}

#[rmcp::tool_router(router = automation_tool_router, vis = "pub")]
impl ZealotServer {
    // Rules

    #[rmcp::tool(description = "List all automation rules. Rules are Lua scripts triggered by events, schedules, or manually.")]
    pub async fn list_rules(&self) -> Result<CallToolResult, McpError> {
        let rules: serde_json::Value = self.client.get("/rule/").await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(pretty(rules))]))
    }

    #[rmcp::tool(description = "Get a single rule by its numeric ID, including the full Lua script.")]
    pub async fn get_rule(
        &self,
        Parameters(p): Parameters<RuleIdParam>,
    ) -> Result<CallToolResult, McpError> {
        let rule: serde_json::Value = self
            .client
            .get(&format!("/rule/{}", p.rule_id))
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(pretty(rule))]))
    }

    #[rmcp::tool(description = "Create a new automation rule with a trigger and Lua script. Trigger shapes: {\"kind\":\"manual\"}, {\"kind\":\"cron\",\"expression\":\"0 9 * * *\"}, {\"kind\":\"on_item_create\"}, {\"kind\":\"on_type_assign\",\"type_name\":\"Goal\"}.")]
    pub async fn create_rule(
        &self,
        Parameters(p): Parameters<CreateRuleParams>,
    ) -> Result<CallToolResult, McpError> {
        let body = json!({
            "name": p.name,
            "description": p.description,
            "trigger": p.trigger,
            "script": p.script,
            "enabled": p.enabled.unwrap_or(true),
        });
        let rule: serde_json::Value = self.client.post("/rule/", &body).await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(pretty(rule))]))
    }

    #[rmcp::tool(description = "Update an existing rule's name, description, trigger, script, or enabled state.")]
    pub async fn update_rule(
        &self,
        Parameters(p): Parameters<UpdateRuleParams>,
    ) -> Result<CallToolResult, McpError> {
        let body = json!({
            "name": p.name,
            "description": p.description,
            "trigger": p.trigger,
            "script": p.script,
            "enabled": p.enabled,
        });
        let rule: serde_json::Value = self
            .client
            .patch(&format!("/rule/{}", p.rule_id), &body)
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(pretty(rule))]))
    }

    #[rmcp::tool(description = "Delete a rule by ID.")]
    pub async fn delete_rule(
        &self,
        Parameters(p): Parameters<RuleIdParam>,
    ) -> Result<CallToolResult, McpError> {
        self.client
            .delete(&format!("/rule/{}", p.rule_id))
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text("rule deleted".to_string())]))
    }

    #[rmcp::tool(description = "Run a rule immediately regardless of its trigger. Returns the execution result including any output or errors from the Lua script.")]
    pub async fn run_rule(
        &self,
        Parameters(p): Parameters<RuleIdParam>,
    ) -> Result<CallToolResult, McpError> {
        let result: serde_json::Value = self
            .client
            .post(&format!("/rule/{}/run", p.rule_id), &json!({}))
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(pretty(result))]))
    }

    // Item Types

    #[rmcp::tool(description = "List all item types defined in this Zealot instance (e.g. Goal, Project, Habit, Task).")]
    pub async fn list_item_types(&self) -> Result<CallToolResult, McpError> {
        let types: serde_json::Value = self.client.get("/item_type/").await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(pretty(types))]))
    }

    #[rmcp::tool(description = "Get a specific item type by its name. Returns the type definition including required attributes.")]
    pub async fn get_item_type_by_name(
        &self,
        Parameters(p): Parameters<ItemTypeNameParam>,
    ) -> Result<CallToolResult, McpError> {
        let t: serde_json::Value = self
            .client
            .get(&format!(
                "/item_type/name/{}",
                urlencoding::encode(&p.name)
            ))
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(pretty(t))]))
    }

    #[rmcp::tool(description = "Create a new item type with a name, optional description, icon, and color.")]
    pub async fn create_item_type(
        &self,
        Parameters(p): Parameters<CreateItemTypeParams>,
    ) -> Result<CallToolResult, McpError> {
        let body = json!({
            "name": p.name,
            "description": p.description,
            "icon": p.icon,
            "color": p.color,
        });
        let t: serde_json::Value =
            self.client.post("/item_type/", &body).await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(pretty(t))]))
    }

    #[rmcp::tool(description = "Update an existing item type's name, description, icon, or color.")]
    pub async fn update_item_type(
        &self,
        Parameters(p): Parameters<UpdateItemTypeParams>,
    ) -> Result<CallToolResult, McpError> {
        let body = json!({
            "name": p.name,
            "description": p.description,
            "icon": p.icon,
            "color": p.color,
        });
        let t: serde_json::Value = self
            .client
            .patch(&format!("/item_type/{}", p.type_id), &body)
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(pretty(t))]))
    }

    #[rmcp::tool(description = "Delete an item type by its numeric ID. Items with this type are not deleted.")]
    pub async fn delete_item_type(
        &self,
        Parameters(p): Parameters<ItemTypeIdParam>,
    ) -> Result<CallToolResult, McpError> {
        self.client
            .delete(&format!("/item_type/{}", p.type_id))
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text("item type deleted".to_string())]))
    }

    // Attribute Kinds

    #[rmcp::tool(description = "List all attribute kind definitions (the schema for custom item attributes).")]
    pub async fn list_attribute_kinds(&self) -> Result<CallToolResult, McpError> {
        let kinds: serde_json::Value = self.client.get("/attribute/").await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(pretty(kinds))]))
    }

    #[rmcp::tool(description = "Get a specific attribute kind by its key (slug).")]
    pub async fn get_attribute_by_key(
        &self,
        Parameters(p): Parameters<AttributeKeyParam>,
    ) -> Result<CallToolResult, McpError> {
        let kind: serde_json::Value = self
            .client
            .get(&format!("/attribute/key/{}", urlencoding::encode(&p.key)))
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(pretty(kind))]))
    }

    #[rmcp::tool(description = "Create a new attribute kind (custom field schema). base_type: text, integer, decimal, date, week, boolean, dropdown, item, list. For dropdown, config: {\"values\":[\"opt1\",\"opt2\"]}.")]
    pub async fn create_attribute_kind(
        &self,
        Parameters(p): Parameters<CreateAttributeKindParams>,
    ) -> Result<CallToolResult, McpError> {
        let body = json!({
            "key": p.key,
            "description": p.description,
            "base_type": p.base_type,
            "config": p.config.unwrap_or(json!({})),
        });
        let kind: serde_json::Value =
            self.client.post("/attribute/", &body).await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(pretty(kind))]))
    }

    #[rmcp::tool(description = "Update an attribute kind's description or config by its numeric ID.")]
    pub async fn update_attribute_kind(
        &self,
        Parameters(p): Parameters<UpdateAttributeKindParams>,
    ) -> Result<CallToolResult, McpError> {
        let body = json!({
            "description": p.description,
            "config": p.config,
        });
        let kind: serde_json::Value = self
            .client
            .patch(&format!("/attribute/id/{}", p.kind_id), &body)
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(pretty(kind))]))
    }
}

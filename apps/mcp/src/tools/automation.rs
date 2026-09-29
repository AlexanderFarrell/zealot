use rmcp::{ErrorData as McpError, handler::server::wrapper::Parameters, model::CallToolResult};
use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::Deserialize;
use serde_json::json;
use zealot_domain::{
    attribute::{AddAttributeKindDto, UpdateAttributeKindDto},
    item_type::{AddItemTypeDto, UpdateItemTypeDto},
    rule::{AddRuleDto, TriggerKind, UpdateRuleDto},
};

use crate::{
    output,
    tools::{ZealotServer, err_ctx},
};

// schemars 1.x generates boolean `true` for serde_json::Value, which the MCP
// SDK's Zod validator rejects. This wrapper emits {"type":"object"} instead.
#[derive(Debug, Deserialize)]
pub struct JsonObject(pub serde_json::Value);

impl JsonSchema for JsonObject {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("JsonObject")
    }
    fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
        schemars::json_schema!({"type": "object"})
    }
}

fn parse_trigger(v: serde_json::Value) -> Result<TriggerKind, McpError> {
    serde_json::from_value(v)
        .map_err(|e| McpError::invalid_params(format!("invalid trigger: {e}"), None))
}

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
    pub trigger: JsonObject,
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
    pub trigger: Option<JsonObject>,
    pub script: Option<String>,
    pub enabled: Option<bool>,
}

// ── Item Types ────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListItemTypesParams {}

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
    pub required_attributes: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateItemTypeParams {
    pub type_id: i64,
    pub name: Option<String>,
    pub description: Option<String>,
    pub required_attributes: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteItemTypeParams {
    pub type_id: i64,
    /// Delete even if items still use this type (default false)
    pub force: Option<bool>,
}

// ── Attribute Kinds ───────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AttributeKeyParam {
    pub key: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateAttributeKindParams {
    /// Unique key (slug) for the attribute (e.g. "due_date", "priority")
    pub key: String,
    pub description: Option<String>,
    /// Base type: text, integer, decimal, date, week, boolean, dropdown, item, list
    pub base_type: String,
    /// Type-specific config (e.g. {"values":["low","medium","high"]} for dropdown)
    pub config: Option<JsonObject>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateAttributeKindParams {
    pub kind_id: i64,
    pub key: Option<String>,
    pub description: Option<String>,
    pub base_type: Option<String>,
    pub config: Option<JsonObject>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteAttributeKindParams {
    pub key: String,
    /// Delete even if items still use this attribute (default false)
    pub force: Option<bool>,
}

#[rmcp::tool_router(router = automation_tool_router, vis = "pub")]
impl ZealotServer {
    // Rules

    #[rmcp::tool(
        description = "List all automation rules. Rules are Lua scripts triggered by events, schedules, or manually. Script bodies are omitted here — use get_rule for the full script."
    )]
    pub async fn list_rules(&self) -> Result<CallToolResult, McpError> {
        let rules = self
            .client
            .list_rules()
            .await
            .map_err(|e| err_ctx("rules", e))?;
        let rows: Vec<_> = rules
            .iter()
            .map(|r| {
                json!({
                    "rule_id": r.rule_id,
                    "name": r.name,
                    "description": r.description,
                    "trigger": r.trigger,
                    "enabled": r.enabled,
                    "script_chars": r.script.chars().count(),
                    "last_run_at": r.last_run_at,
                    "last_error": r.last_error,
                })
            })
            .collect();
        Ok(output::json_result(&rows))
    }

    #[rmcp::tool(
        description = "Get a single rule by its numeric ID, including the full Lua script."
    )]
    pub async fn get_rule(
        &self,
        Parameters(p): Parameters<RuleIdParam>,
    ) -> Result<CallToolResult, McpError> {
        let rule = self
            .client
            .get_rule(p.rule_id)
            .await
            .map_err(|e| err_ctx(&format!("rule #{}", p.rule_id), e))?;
        Ok(output::json_result(&rule))
    }

    #[rmcp::tool(
        description = "Create a new automation rule with a trigger and Lua script. Trigger shapes: {\"kind\":\"manual\"}, {\"kind\":\"cron\",\"expression\":\"0 9 * * *\"}, {\"kind\":\"on_item_create\"}, {\"kind\":\"on_type_assign\",\"type_name\":\"Goal\"}."
    )]
    pub async fn create_rule(
        &self,
        Parameters(p): Parameters<CreateRuleParams>,
    ) -> Result<CallToolResult, McpError> {
        let dto = AddRuleDto {
            name: p.name,
            description: p.description,
            trigger: parse_trigger(p.trigger.0)?,
            script: p.script,
            enabled: p.enabled,
        };
        let rule = self
            .client
            .add_rule(&dto)
            .await
            .map_err(|e| err_ctx("new rule", e))?;
        Ok(output::json_result(&rule))
    }

    #[rmcp::tool(
        description = "Update an existing rule's name, description, trigger, script, or enabled state."
    )]
    pub async fn update_rule(
        &self,
        Parameters(p): Parameters<UpdateRuleParams>,
    ) -> Result<CallToolResult, McpError> {
        let trigger = p.trigger.map(|t| parse_trigger(t.0)).transpose()?;
        let dto = UpdateRuleDto {
            name: p.name,
            description: p.description,
            trigger,
            script: p.script,
            enabled: p.enabled,
        };
        let rule = self
            .client
            .update_rule(p.rule_id, &dto)
            .await
            .map_err(|e| err_ctx(&format!("rule #{}", p.rule_id), e))?;
        Ok(output::json_result(&rule))
    }

    #[rmcp::tool(description = "Delete a rule by ID.")]
    pub async fn delete_rule(
        &self,
        Parameters(p): Parameters<RuleIdParam>,
    ) -> Result<CallToolResult, McpError> {
        self.client
            .delete_rule(p.rule_id)
            .await
            .map_err(|e| err_ctx(&format!("rule #{}", p.rule_id), e))?;
        Ok(output::json_result(&json!({"deleted": p.rule_id})))
    }

    #[rmcp::tool(
        description = "Run a rule immediately regardless of its trigger. Returns the execution result including any output or errors from the Lua script."
    )]
    pub async fn run_rule(
        &self,
        Parameters(p): Parameters<RuleIdParam>,
    ) -> Result<CallToolResult, McpError> {
        let result = self
            .client
            .run_rule(p.rule_id)
            .await
            .map_err(|e| err_ctx(&format!("rule #{}", p.rule_id), e))?;
        Ok(output::json_result(&result))
    }

    // Item Types

    #[rmcp::tool(
        description = "List all item types defined in this Zealot instance, with item counts and required-attribute counts (e.g. Goal, Project, Habit, Task)."
    )]
    pub async fn list_item_types(
        &self,
        Parameters(_p): Parameters<ListItemTypesParams>,
    ) -> Result<CallToolResult, McpError> {
        let types = self
            .client
            .item_type_summaries()
            .await
            .map_err(|e| err_ctx("item types", e))?;
        Ok(output::json_result(&types))
    }

    #[rmcp::tool(
        description = "Get a specific item type by its name. Returns the type definition including required attributes."
    )]
    pub async fn get_item_type(
        &self,
        Parameters(p): Parameters<ItemTypeNameParam>,
    ) -> Result<CallToolResult, McpError> {
        let t = self
            .client
            .get_item_type_by_name(&p.name)
            .await
            .map_err(|e| err_ctx(&format!("item type '{}'", p.name), e))?;
        Ok(output::json_result(&t))
    }

    #[rmcp::tool(
        description = "Create a new item type with a name, optional description, and required attribute keys."
    )]
    pub async fn create_item_type(
        &self,
        Parameters(p): Parameters<CreateItemTypeParams>,
    ) -> Result<CallToolResult, McpError> {
        let dto = AddItemTypeDto {
            name: p.name,
            description: p.description.unwrap_or_default(),
            required_attributes: p.required_attributes.unwrap_or_default(),
        };
        let t = self
            .client
            .add_item_type(&dto)
            .await
            .map_err(|e| err_ctx("new item type", e))?;
        Ok(output::json_result(&t))
    }

    #[rmcp::tool(
        description = "Update an existing item type's name, description, or required attributes."
    )]
    pub async fn update_item_type(
        &self,
        Parameters(p): Parameters<UpdateItemTypeParams>,
    ) -> Result<CallToolResult, McpError> {
        let dto = UpdateItemTypeDto {
            type_id: p.type_id,
            name: p.name,
            description: p.description,
            required_attributes: p.required_attributes,
        };
        let t = self
            .client
            .update_item_type(&dto)
            .await
            .map_err(|e| err_ctx(&format!("item type #{}", p.type_id), e))?;
        Ok(output::json_result(&t))
    }

    #[rmcp::tool(
        description = "Delete an item type by its numeric ID. Items with this type are not deleted. Pass force:true to delete even if items still use it."
    )]
    pub async fn delete_item_type(
        &self,
        Parameters(p): Parameters<DeleteItemTypeParams>,
    ) -> Result<CallToolResult, McpError> {
        self.client
            .delete_item_type(p.type_id, p.force.unwrap_or(false))
            .await
            .map_err(|e| err_ctx(&format!("item type #{}", p.type_id), e))?;
        Ok(output::json_result(&json!({"deleted": p.type_id})))
    }

    // Attribute Kinds

    #[rmcp::tool(
        description = "List all attribute kind definitions (the schema for custom item attributes)."
    )]
    pub async fn list_attribute_kinds(&self) -> Result<CallToolResult, McpError> {
        let kinds = self
            .client
            .list_attribute_kinds()
            .await
            .map_err(|e| err_ctx("attribute kinds", e))?;
        Ok(output::json_result(&kinds))
    }

    #[rmcp::tool(description = "Get a specific attribute kind by its key (slug).")]
    pub async fn get_attribute_kind(
        &self,
        Parameters(p): Parameters<AttributeKeyParam>,
    ) -> Result<CallToolResult, McpError> {
        let kind = self
            .client
            .get_attribute_kind_by_key(&p.key)
            .await
            .map_err(|e| err_ctx(&format!("attribute kind '{}'", p.key), e))?;
        Ok(output::json_result(&kind))
    }

    #[rmcp::tool(
        description = "Create a new attribute kind (custom field schema). base_type: text, integer, decimal, date, week, boolean, dropdown, item, list. For dropdown, config: {\"values\":[\"opt1\",\"opt2\"]}."
    )]
    pub async fn create_attribute_kind(
        &self,
        Parameters(p): Parameters<CreateAttributeKindParams>,
    ) -> Result<CallToolResult, McpError> {
        let dto = AddAttributeKindDto {
            key: p.key,
            description: p.description.unwrap_or_default(),
            base_type: p.base_type,
            config: p.config.map(|c| c.0).unwrap_or(json!({})),
        };
        let kind = self
            .client
            .add_attribute_kind(&dto)
            .await
            .map_err(|e| err_ctx("new attribute kind", e))?;
        Ok(output::json_result(&kind))
    }

    #[rmcp::tool(
        description = "Update an attribute kind's key, description, base type, or config by its numeric ID."
    )]
    pub async fn update_attribute_kind(
        &self,
        Parameters(p): Parameters<UpdateAttributeKindParams>,
    ) -> Result<CallToolResult, McpError> {
        let dto = UpdateAttributeKindDto {
            kind_id: p.kind_id,
            key: p.key,
            description: p.description,
            base_type: p.base_type,
            config: p.config.map(|c| c.0),
        };
        let kind = self
            .client
            .update_attribute_kind(&dto)
            .await
            .map_err(|e| err_ctx(&format!("attribute kind #{}", p.kind_id), e))?;
        Ok(output::json_result(&kind))
    }

    #[rmcp::tool(
        description = "Delete an attribute kind by its key. Pass force:true to delete even if items still use it."
    )]
    pub async fn delete_attribute_kind(
        &self,
        Parameters(p): Parameters<DeleteAttributeKindParams>,
    ) -> Result<CallToolResult, McpError> {
        self.client
            .delete_attribute_kind(&p.key, p.force.unwrap_or(false))
            .await
            .map_err(|e| err_ctx(&format!("attribute kind '{}'", p.key), e))?;
        Ok(output::json_result(&json!({"deleted": p.key})))
    }
}

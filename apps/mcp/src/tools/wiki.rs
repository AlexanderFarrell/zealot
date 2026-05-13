use rmcp::{
    model::{CallToolResult, Content},
    ErrorData as McpError,
    handler::server::wrapper::Parameters,
};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use crate::tools::{ZealotServer, api_err};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ItemIdParam {
    /// Numeric item ID
    pub id: i64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListItemsParams {
    /// Optional: filter by item type name (e.g. "Goal", "Project")
    pub type_filter: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RecentItemsParams {
    /// Maximum number of items to return (default 30)
    pub limit: Option<i64>,
    /// Offset for pagination (default 0)
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchItemsParams {
    /// Search term to match against item titles
    pub term: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetItemByTitleParams {
    /// Exact title of the item
    pub title: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateItemParams {
    /// Title of the new item
    pub title: String,
    /// Content body (ZealotScript / markdown)
    pub content: String,
    /// Optional attributes as a JSON object (e.g. {"priority": "high", "due": "2025-06-01"})
    pub attributes: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateItemParams {
    /// Numeric item ID
    pub id: i64,
    /// New title (optional)
    pub title: Option<String>,
    /// New content body (optional)
    pub content: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetAttributesParams {
    /// Numeric item ID
    pub id: i64,
    /// JSON object of attribute key→value pairs to set
    pub attributes: serde_json::Value,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteAttributeParams {
    /// Numeric item ID
    pub id: i64,
    /// Attribute key to delete
    pub key: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AssignTypeParams {
    /// Numeric item ID
    pub item_id: i64,
    /// Name of the type to assign or unassign (e.g. "Goal", "Project")
    pub type_name: String,
}

fn pretty(v: serde_json::Value) -> String {
    serde_json::to_string_pretty(&v).unwrap_or_default()
}

#[rmcp::tool_router(router = wiki_tool_router, vis = "pub")]
impl ZealotServer {
    #[rmcp::tool(description = "List root-level wiki items. Optionally filter by type name (e.g. 'Goal', 'Project').")]
    pub async fn list_items(
        &self,
        Parameters(p): Parameters<ListItemsParams>,
    ) -> Result<CallToolResult, McpError> {
        let path = match &p.type_filter {
            Some(t) => format!("/item/?type={}", urlencoding::encode(t)),
            None => "/item/".to_string(),
        };
        let items: serde_json::Value = self.client.get(&path).await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(pretty(items))]))
    }

    #[rmcp::tool(description = "List recently modified items. Returns up to `limit` items starting from `offset`.")]
    pub async fn list_recent_items(
        &self,
        Parameters(p): Parameters<RecentItemsParams>,
    ) -> Result<CallToolResult, McpError> {
        let limit = p.limit.unwrap_or(30);
        let offset = p.offset.unwrap_or(0);
        let items: serde_json::Value = self
            .client
            .get(&format!("/item/recent?limit={limit}&offset={offset}"))
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(pretty(items))]))
    }

    #[rmcp::tool(description = "Search items by title keyword. Returns matching items sorted by relevance.")]
    pub async fn search_items(
        &self,
        Parameters(p): Parameters<SearchItemsParams>,
    ) -> Result<CallToolResult, McpError> {
        let items: serde_json::Value = self
            .client
            .get(&format!(
                "/item/search?term={}",
                urlencoding::encode(&p.term)
            ))
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(pretty(items))]))
    }

    #[rmcp::tool(description = "Get a specific item by its numeric ID. Returns full item including attributes, types, and links.")]
    pub async fn get_item(
        &self,
        Parameters(p): Parameters<ItemIdParam>,
    ) -> Result<CallToolResult, McpError> {
        let item: serde_json::Value = self
            .client
            .get(&format!("/item/id/{}", p.id))
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(pretty(item))]))
    }

    #[rmcp::tool(description = "Get a specific item by its exact title. Returns full item including attributes, types, and links.")]
    pub async fn get_item_by_title(
        &self,
        Parameters(p): Parameters<GetItemByTitleParams>,
    ) -> Result<CallToolResult, McpError> {
        let item: serde_json::Value = self
            .client
            .get(&format!(
                "/item/title/{}",
                urlencoding::encode(&p.title)
            ))
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(pretty(item))]))
    }

    #[rmcp::tool(description = "Get all child items of a given item. Returns items that are children (sub-items) of the specified parent.")]
    pub async fn get_children(
        &self,
        Parameters(p): Parameters<ItemIdParam>,
    ) -> Result<CallToolResult, McpError> {
        let items: serde_json::Value = self
            .client
            .get(&format!("/item/children/{}", p.id))
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(pretty(items))]))
    }

    #[rmcp::tool(description = "Get all items related to a given item (linked items, blocked by, tagged, etc.).")]
    pub async fn get_related_items(
        &self,
        Parameters(p): Parameters<ItemIdParam>,
    ) -> Result<CallToolResult, McpError> {
        let items: serde_json::Value = self
            .client
            .get(&format!("/item/related/{}", p.id))
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(pretty(items))]))
    }

    #[rmcp::tool(description = "Create a new wiki item with a title and content body. Optionally provide attributes as a JSON object.")]
    pub async fn create_item(
        &self,
        Parameters(p): Parameters<CreateItemParams>,
    ) -> Result<CallToolResult, McpError> {
        let body = json!({
            "title": p.title,
            "content": p.content,
            "attributes": p.attributes.unwrap_or(json!({})),
        });
        let item: serde_json::Value = self.client.post("/item/", &body).await.map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(pretty(item))]))
    }

    #[rmcp::tool(description = "Update an existing item's title and/or content by ID. Only provided fields are changed.")]
    pub async fn update_item(
        &self,
        Parameters(p): Parameters<UpdateItemParams>,
    ) -> Result<CallToolResult, McpError> {
        let body = json!({ "title": p.title, "content": p.content });
        let item: serde_json::Value = self
            .client
            .patch(&format!("/{}", p.id), &body)
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(pretty(item))]))
    }

    #[rmcp::tool(description = "Delete an item by ID. This is permanent and cannot be undone.")]
    pub async fn delete_item(
        &self,
        Parameters(p): Parameters<ItemIdParam>,
    ) -> Result<CallToolResult, McpError> {
        self.client
            .delete(&format!("/{}", p.id))
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text("deleted".to_string())]))
    }

    #[rmcp::tool(description = "Set one or more attributes on an item. Pass a JSON object of key→value pairs. Existing keys are overwritten, others are preserved.")]
    pub async fn set_item_attributes(
        &self,
        Parameters(p): Parameters<SetAttributesParams>,
    ) -> Result<CallToolResult, McpError> {
        let _: serde_json::Value = self
            .client
            .patch(&format!("/{}/attr", p.id), &p.attributes)
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text("attributes updated".to_string())]))
    }

    #[rmcp::tool(description = "Delete a single attribute from an item by its key.")]
    pub async fn delete_item_attribute(
        &self,
        Parameters(p): Parameters<DeleteAttributeParams>,
    ) -> Result<CallToolResult, McpError> {
        self.client
            .delete(&format!("/{}/attr/{}", p.id, urlencoding::encode(&p.key)))
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text("attribute deleted".to_string())]))
    }

    #[rmcp::tool(description = "Assign an item type to an item (e.g. assign 'Goal' type to an item). An item can have multiple types.")]
    pub async fn assign_item_type(
        &self,
        Parameters(p): Parameters<AssignTypeParams>,
    ) -> Result<CallToolResult, McpError> {
        let _: serde_json::Value = self
            .client
            .post(
                &format!(
                    "/{}/assign_type/{}",
                    p.item_id,
                    urlencoding::encode(&p.type_name)
                ),
                &json!({}),
            )
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(format!(
            "type '{}' assigned",
            p.type_name
        ))]))
    }

    #[rmcp::tool(description = "Remove a type from an item. The item itself is not deleted.")]
    pub async fn unassign_item_type(
        &self,
        Parameters(p): Parameters<AssignTypeParams>,
    ) -> Result<CallToolResult, McpError> {
        self.client
            .delete(&format!(
                "/{}/assign_type/{}",
                p.item_id,
                urlencoding::encode(&p.type_name)
            ))
            .await
            .map_err(api_err)?;
        Ok(CallToolResult::success(vec![Content::text(format!(
            "type '{}' removed",
            p.type_name
        ))]))
    }
}

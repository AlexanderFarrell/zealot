use std::collections::HashMap;

use rmcp::{ErrorData as McpError, handler::server::wrapper::Parameters, model::CallToolResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use zealot_domain::{
    attribute::AttributeFilterDto,
    item::{AddItemDto, ItemDto, ItemLinkDto, SearchScope, UpdateItemDto},
};

use crate::{
    output::{self, DEFAULT_LIMIT, Detail},
    tools::{ZealotServer, err_ctx},
};

const SEARCH_DEFAULT_LIMIT: i64 = 20;
const MAX_LIMIT: i64 = 100;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ItemRefParam {
    /// Item ID or exact title
    pub item: String,
    /// Output detail: "meta", "summary", or "full" (default "full")
    #[serde(default = "Detail::full")]
    pub detail: Detail,
}

impl Detail {
    fn full() -> Detail {
        Detail::Full
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ItemOutlineParams {
    /// Item ID or exact title
    pub item: String,
}

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum BrowseMode {
    /// Root-level items (optionally filtered by type_filter)
    Root,
    /// Most recently modified items
    Recent,
    /// A random sample of items
    Random,
    /// Items with the most views
    MostViewed,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BrowseItemsParams {
    /// Which set of items to browse
    pub mode: BrowseMode,
    /// Filter by item type name — only used when mode is "root"
    pub type_filter: Option<String>,
    /// Max results (default 50, max 100)
    pub limit: Option<i64>,
    /// Offset for pagination (ignored for "random")
    pub offset: Option<i64>,
    /// Output detail: "meta", "summary" (default), or "full"
    #[serde(default)]
    pub detail: Detail,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchItemsParams {
    /// Search term to match against item titles, body content, or headings
    pub term: String,
    /// Max results (default 20, max 100)
    pub limit: Option<i64>,
    /// Offset for pagination (default 0)
    pub offset: Option<i64>,
    /// Search scope: "title" (default), "content" (body text), or "heading"
    pub scope: Option<String>,
    /// If true, treat `term` as a case-insensitive regular expression
    pub regex: Option<bool>,
    /// Output detail: "meta", "summary" (default), or "full"
    #[serde(default)]
    pub detail: Detail,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct AttributeFilterParam {
    /// Attribute key/name to filter on
    pub key: String,
    /// Operator: eq, ne, gt, lt, gte, lte, or ilike
    pub op: String,
    /// JSON value to compare against
    pub value: serde_json::Value,
    /// List matching mode: any (default), all, or none
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub list_mode: Option<String>,
}

impl From<AttributeFilterParam> for AttributeFilterDto {
    fn from(p: AttributeFilterParam) -> Self {
        AttributeFilterDto {
            key: p.key,
            op: p.op,
            value: p.value,
            list_mode: p.list_mode.unwrap_or_else(|| "any".to_string()),
        }
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FilterItemsParams {
    /// Attribute filters. All filters are ANDed together.
    pub filters: Vec<AttributeFilterParam>,
    /// Max results (default 50, max 100)
    pub limit: Option<i64>,
    /// Offset for pagination (default 0)
    pub offset: Option<i64>,
    /// Output detail: "meta", "summary" (default), or "full"
    #[serde(default)]
    pub detail: Detail,
}

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum LinkDirection {
    /// Sub-items of this item
    Children,
    /// All items linked to this item (any relationship)
    Related,
    /// Items that link to this item
    Backlinks,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LinkedItemsParams {
    /// Item ID or exact title
    pub item: String,
    /// Which set of linked items to return
    pub direction: LinkDirection,
    /// Output detail: "meta", "summary" (default), or "full"
    #[serde(default)]
    pub detail: Detail,
    /// Max results (default 50, max 100)
    pub limit: Option<i64>,
    /// Offset for pagination (default 0)
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LinkParam {
    /// ID of the other item
    pub other_item_id: i64,
    /// Relationship label, e.g. "parent", "blocks", "tag", "topic", or any
    /// user-defined attribute kind key
    pub relationship: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateItemParams {
    /// Title of the new item
    pub title: String,
    /// Content body (ZealotScript / markdown)
    pub content: Option<String>,
    /// Attributes as a JSON object (e.g. {"priority": "high", "due": "2025-06-01"})
    pub attributes: Option<HashMap<String, serde_json::Value>>,
    /// Type names to assign at creation
    pub types: Option<Vec<String>>,
    /// Relationship links to other items
    pub links: Option<Vec<LinkParam>>,
    /// Parent item (ID or title) — sets the "Parent" attribute, same as the
    /// wiki's parent/child hierarchy
    pub parent: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateItemParams {
    /// Item ID or exact title
    pub item: String,
    /// New title (optional)
    pub title: Option<String>,
    /// New content body (optional)
    pub content: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AppendToItemParams {
    /// Item ID or exact title
    pub item: String,
    /// Text to append as a new paragraph at the end of the item's content
    pub text: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ItemOnlyParams {
    /// Item ID or exact title
    pub item: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RenameAttributeParam {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateAttributesParams {
    /// Item ID or exact title
    pub item: String,
    /// Attribute key→value pairs to set (existing keys are overwritten)
    pub set: Option<HashMap<String, serde_json::Value>>,
    /// Attribute keys to delete
    pub remove: Option<Vec<String>>,
    /// Attribute keys to rename, applied after `set` and before `remove`
    pub rename: Option<Vec<RenameAttributeParam>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AssignTypeParams {
    /// Item ID or exact title
    pub item: String,
    /// Name of the type to assign or unassign (e.g. "Goal", "Project")
    pub type_name: String,
}

fn clamp_limit(limit: Option<i64>, default: i64) -> i64 {
    limit.unwrap_or(default).clamp(1, MAX_LIMIT)
}

#[rmcp::tool_router(router = wiki_tool_router, vis = "pub")]
impl ZealotServer {
    #[rmcp::tool(
        description = "Get a specific item by ID or exact title. Defaults to full content — pass detail:\"summary\" or \"meta\" for a lighter response."
    )]
    pub async fn get_item(
        &self,
        Parameters(p): Parameters<ItemRefParam>,
    ) -> Result<CallToolResult, McpError> {
        let item = self.resolve_item(&p.item).await?;
        Ok(match p.detail {
            Detail::Full => output::json_result(&item),
            d => output::json_result(&output::ItemSummary::project(&item, d)),
        })
    }

    #[rmcp::tool(
        description = "Get the heading outline of an item's content (all Markdown headings with their levels), plus its attribute keys and content length. Use this to understand the structure of a long item before deciding whether to fetch full content."
    )]
    pub async fn get_item_outline(
        &self,
        Parameters(p): Parameters<ItemOutlineParams>,
    ) -> Result<CallToolResult, McpError> {
        let item = self.resolve_item(&p.item).await?;
        let headings: Vec<serde_json::Value> = item
            .content
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim_start();
                let level = trimmed.chars().take_while(|&c| c == '#').count();
                if (1..=6).contains(&level) && trimmed.as_bytes().get(level) == Some(&b' ') {
                    Some(json!({
                        "level": level,
                        "text": trimmed[level..].trim(),
                    }))
                } else {
                    None
                }
            })
            .collect();
        let attribute_keys: Vec<String> = item
            .attributes
            .as_object()
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default();
        Ok(output::json_result(&json!({
            "item_id": item.item_id,
            "title": item.title,
            "content_chars": item.content.chars().count(),
            "attribute_keys": attribute_keys,
            "headings": headings,
        })))
    }

    #[rmcp::tool(
        description = "Browse items without a search term. mode: \"root\" (root-level items, optionally filtered by type_filter), \"recent\" (most recently modified), \"random\" (random sample), or \"most_viewed\" (most-viewed items — rows are {item_id, title, view_count} regardless of detail). Returns compact summaries by default; pass detail:\"full\" for complete content."
    )]
    pub async fn browse_items(
        &self,
        Parameters(p): Parameters<BrowseItemsParams>,
    ) -> Result<CallToolResult, McpError> {
        let limit = clamp_limit(p.limit, DEFAULT_LIMIT);
        let offset = p.offset.unwrap_or(0).max(0);
        match p.mode {
            BrowseMode::Root => {
                let items = self
                    .client
                    .list_items(p.type_filter.as_deref())
                    .await
                    .map_err(|e| err_ctx("items", e))?;
                Ok(slice_item_page(items, p.detail, offset, limit))
            }
            BrowseMode::Recent => {
                let items = self
                    .client
                    .recent_items(limit, offset)
                    .await
                    .map_err(|e| err_ctx("recent items", e))?;
                Ok(output::item_page(items, p.detail, offset, limit))
            }
            BrowseMode::Random => {
                let items = self
                    .client
                    .random_items(limit as usize)
                    .await
                    .map_err(|e| err_ctx("random items", e))?;
                Ok(output::item_page(items, p.detail, 0, limit))
            }
            BrowseMode::MostViewed => {
                let rows = self
                    .client
                    .most_viewed(limit)
                    .await
                    .map_err(|e| err_ctx("most-viewed items", e))?;
                Ok(output::json_result(&rows))
            }
        }
    }

    #[rmcp::tool(
        description = "Search items by title, body content, or headings. `scope` controls what is searched: \"title\" (default), \"content\" (body text), or \"heading\". Set `regex: true` to treat `term` as a case-insensitive regular expression. Results include `match_scope` and a `snippet`. Returns compact summaries by default; pass detail:\"full\" for complete content. Example: `{\"term\": \"architecture\", \"scope\": \"content\"}`."
    )]
    pub async fn search_items(
        &self,
        Parameters(p): Parameters<SearchItemsParams>,
    ) -> Result<CallToolResult, McpError> {
        let limit = clamp_limit(p.limit, SEARCH_DEFAULT_LIMIT);
        let offset = p.offset.unwrap_or(0).max(0);
        let scope = match p.scope.as_deref() {
            Some("content") => SearchScope::Content,
            Some("heading") => SearchScope::Heading,
            _ => SearchScope::Title,
        };
        let hits = self
            .client
            .search_items(&p.term, scope, p.regex.unwrap_or(false), limit, offset)
            .await
            .map_err(|e| err_ctx("search", e))?;
        Ok(output::search_page(hits, p.detail, offset, limit))
    }

    #[rmcp::tool(
        description = "Filter items by attribute values. Pass `filters` as an array of {key, op, value, list_mode?}; supported ops are eq, ne, gt, lt, gte, lte, and ilike. All filters are ANDed. Returns compact summaries by default; pass detail:\"full\" for complete content."
    )]
    pub async fn filter_items(
        &self,
        Parameters(p): Parameters<FilterItemsParams>,
    ) -> Result<CallToolResult, McpError> {
        let limit = clamp_limit(p.limit, DEFAULT_LIMIT);
        let offset = p.offset.unwrap_or(0).max(0);
        let filters: Vec<AttributeFilterDto> = p.filters.into_iter().map(Into::into).collect();
        let items = self
            .client
            .filter_items(&filters, limit, offset)
            .await
            .map_err(|e| err_ctx("filtered items", e))?;
        Ok(output::item_page(items, p.detail, offset, limit))
    }

    #[rmcp::tool(
        description = "List items linked to an item. direction: \"children\" (sub-items), \"related\" (all linked items), or \"backlinks\" (items that link here). Returns compact summaries by default; pass detail:\"full\" for complete content."
    )]
    pub async fn get_linked_items(
        &self,
        Parameters(p): Parameters<LinkedItemsParams>,
    ) -> Result<CallToolResult, McpError> {
        let limit = clamp_limit(p.limit, DEFAULT_LIMIT);
        let offset = p.offset.unwrap_or(0).max(0);
        let item = self.resolve_item(&p.item).await?;
        let items = match p.direction {
            LinkDirection::Children => self.client.get_children(item.item_id).await,
            LinkDirection::Related => self.client.get_related(item.item_id).await,
            LinkDirection::Backlinks => self.client.get_backlinks(item.item_id).await,
        }
        .map_err(|e| err_ctx(&format!("links of item #{}", item.item_id), e))?;
        Ok(slice_item_page(items, p.detail, offset, limit))
    }

    #[rmcp::tool(
        description = "Create a new wiki item. `content` is ZealotScript/markdown. `attributes` is a JSON object of key→value pairs. `types` assigns item types at creation. `links` sets relationships to other items by ID. `parent` (ID or title) sets the item's Parent attribute, placing it in the wiki hierarchy."
    )]
    pub async fn create_item(
        &self,
        Parameters(p): Parameters<CreateItemParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut attributes = p.attributes.unwrap_or_default();
        if let Some(parent) = &p.parent {
            let parent = self.resolve_item(parent).await?;
            attributes.insert("Parent".to_string(), json!(parent.item_id));
        }
        let dto = AddItemDto {
            title: p.title,
            content: p.content.unwrap_or_default(),
            attributes: (!attributes.is_empty()).then_some(attributes),
            types: p.types,
            links: p.links.map(|links| {
                links
                    .into_iter()
                    .map(|l| ItemLinkDto {
                        other_item_id: l.other_item_id,
                        relationship: l.relationship,
                    })
                    .collect()
            }),
        };
        let item = self
            .client
            .add_item(&dto)
            .await
            .map_err(|e| err_ctx("new item", e))?;
        Ok(output::json_result(&item))
    }

    #[rmcp::tool(
        description = "Update an existing item's title and/or content. Only provided fields are changed."
    )]
    pub async fn update_item(
        &self,
        Parameters(p): Parameters<UpdateItemParams>,
    ) -> Result<CallToolResult, McpError> {
        let existing = self.resolve_item(&p.item).await?;
        let dto = UpdateItemDto {
            item_id: existing.item_id,
            title: p.title,
            content: p.content,
            attributes: None,
            links: None,
        };
        let item = self
            .client
            .update_item(&dto)
            .await
            .map_err(|e| err_ctx(&format!("item #{}", existing.item_id), e))?;
        Ok(output::json_result(&item))
    }

    #[rmcp::tool(
        description = "Append text as a new paragraph at the end of an item's content. Useful for log-style notes without needing to fetch and resend the full body."
    )]
    pub async fn append_to_item(
        &self,
        Parameters(p): Parameters<AppendToItemParams>,
    ) -> Result<CallToolResult, McpError> {
        let existing = self.resolve_item(&p.item).await?;
        let mut content = existing.content.clone();
        if !content.is_empty() && !content.ends_with('\n') {
            content.push('\n');
        }
        content.push_str(&p.text);
        content.push('\n');
        let dto = UpdateItemDto {
            item_id: existing.item_id,
            title: None,
            content: Some(content),
            attributes: None,
            links: None,
        };
        let item = self
            .client
            .update_item(&dto)
            .await
            .map_err(|e| err_ctx(&format!("item #{}", existing.item_id), e))?;
        Ok(output::json_result(&json!({
            "item_id": item.item_id,
            "title": item.title,
            "content_chars": item.content.chars().count(),
        })))
    }

    #[rmcp::tool(description = "Delete an item. This is permanent and cannot be undone.")]
    pub async fn delete_item(
        &self,
        Parameters(p): Parameters<ItemOnlyParams>,
    ) -> Result<CallToolResult, McpError> {
        let existing = self.resolve_item(&p.item).await?;
        self.client
            .delete_item(existing.item_id)
            .await
            .map_err(|e| err_ctx(&format!("item #{}", existing.item_id), e))?;
        Ok(output::json_result(&json!({"deleted": existing.item_id})))
    }

    #[rmcp::tool(
        description = "Set, rename, and/or remove attributes on an item in one call. `set` overwrites/creates keys, `rename` renames keys (applied after set), `remove` deletes keys (applied last)."
    )]
    pub async fn update_item_attributes(
        &self,
        Parameters(p): Parameters<UpdateAttributesParams>,
    ) -> Result<CallToolResult, McpError> {
        let existing = self.resolve_item(&p.item).await?;
        let resource = format!("item #{}", existing.item_id);

        if let Some(set) = p.set {
            self.client
                .set_item_attributes(existing.item_id, &set)
                .await
                .map_err(|e| err_ctx(&resource, e))?;
        }
        for r in p.rename.into_iter().flatten() {
            self.client
                .rename_item_attribute(existing.item_id, &r.from, &r.to)
                .await
                .map_err(|e| err_ctx(&resource, e))?;
        }
        for key in p.remove.into_iter().flatten() {
            self.client
                .delete_item_attribute(existing.item_id, &key)
                .await
                .map_err(|e| err_ctx(&resource, e))?;
        }
        Ok(output::json_result(
            &json!({"item_id": existing.item_id, "status": "attributes updated"}),
        ))
    }

    #[rmcp::tool(
        description = "Assign an item type to an item (e.g. assign 'Goal' type to an item). An item can have multiple types."
    )]
    pub async fn assign_item_type(
        &self,
        Parameters(p): Parameters<AssignTypeParams>,
    ) -> Result<CallToolResult, McpError> {
        let existing = self.resolve_item(&p.item).await?;
        self.client
            .assign_type(existing.item_id, &p.type_name)
            .await
            .map_err(|e| err_ctx(&format!("item #{}", existing.item_id), e))?;
        Ok(output::json_result(
            &json!({"item_id": existing.item_id, "type": p.type_name, "status": "assigned"}),
        ))
    }

    #[rmcp::tool(description = "Remove a type from an item. The item itself is not deleted.")]
    pub async fn unassign_item_type(
        &self,
        Parameters(p): Parameters<AssignTypeParams>,
    ) -> Result<CallToolResult, McpError> {
        let existing = self.resolve_item(&p.item).await?;
        self.client
            .unassign_type(existing.item_id, &p.type_name)
            .await
            .map_err(|e| err_ctx(&format!("item #{}", existing.item_id), e))?;
        Ok(output::json_result(
            &json!({"item_id": existing.item_id, "type": p.type_name, "status": "removed"}),
        ))
    }

    #[rmcp::tool(
        description = "Rebuild the derived link index from item attributes (Parent, and other item-typed attributes). Run this if links/backlinks look stale after bulk attribute edits."
    )]
    pub async fn rebuild_links(&self) -> Result<CallToolResult, McpError> {
        let result = self
            .client
            .rebuild_links()
            .await
            .map_err(|e| err_ctx("rebuild-links", e))?;
        Ok(output::json_result(&result))
    }
}

/// `list_items` (root/type-filtered) is unbounded server-side; slice + project client-side.
fn slice_item_page(items: Vec<ItemDto>, detail: Detail, offset: i64, limit: i64) -> CallToolResult {
    let start = offset.max(0) as usize;
    let end = start.saturating_add(limit.max(0) as usize);
    let sliced: Vec<ItemDto> = items.into_iter().skip(start).take(end - start).collect();
    output::item_page(sliced, detail, offset, limit)
}

use std::collections::HashMap;

use rmcp::{ErrorData as McpError, handler::server::wrapper::Parameters, model::CallToolResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use zealot_domain::{
    attribute::AttributeFilterDto,
    comment::AddCommentDto,
    item::{
        AddItemDto, ItemDto, ItemLinkDto, ItemQueryDto, ItemSortDto, SearchScope, UpdateItemDto,
    },
    item_content::{
        self, ContentEdit, ContentEditDto, ContentError, ContentWriteDto, GrepQueryDto,
    },
};

use crate::{
    output::{self, DEFAULT_LIMIT, Detail, Shape},
    tools::{ZealotServer, err_ctx, planner::now_timestamp},
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

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReadItemParams {
    /// Item ID or exact title
    pub item: String,
    /// Heading whose section to read, as its text ("Goal") or with its level
    /// ("## Goal"). The section runs to the next heading of the same or
    /// higher level. Mutually exclusive with `lines`
    pub section: Option<String>,
    /// 1-based line range: "12", "12-40", or "12-" (to the end). Mutually
    /// exclusive with `section`
    pub lines: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum BrowseMode {
    /// Root-level items; with `type_filter`, every item of that type at any depth
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
    /// Include archived items (default false).
    #[serde(default)]
    pub include_archived: bool,
    /// Which set of items to browse
    pub mode: BrowseMode,
    /// Item type name. Only used when mode is "root": lists every item of this
    /// type, whether or not it has a parent
    pub type_filter: Option<String>,
    /// Sort order. Only used when mode is "root" with `type_filter`
    pub sort: Option<SortParam>,
    /// Max results (default 50, max 100)
    pub limit: Option<i64>,
    /// Offset for pagination (ignored for "random")
    pub offset: Option<i64>,
    /// Output detail: "meta", "summary" (default), or "full"
    #[serde(default)]
    pub detail: Detail,
    /// Return only these attributes per item (plus `item_id`, `title`, `types`),
    /// overriding `detail`. For example ["TicketID", "Status"]
    pub fields: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchItemsParams {
    /// Include archived items (default false).
    #[serde(default)]
    pub include_archived: bool,
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
    /// Return only these attributes per item (plus `item_id`, `title`, `types`),
    /// overriding `detail`. For example ["TicketID", "Status"]
    pub fields: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct AttributeFilterParam {
    /// Attribute key/name to filter on
    pub key: String,
    /// Operator: eq, ne, gt, lt, gte, lte, ilike, exists, or missing. `ilike` is a
    /// case-insensitive substring match (`%value%`) on text values only. `exists`
    /// and `missing` test whether the item has the attribute at all (no value needed)
    pub op: String,
    /// JSON value, or an array of values, to compare against. Item-valued
    /// attributes (such as Parent) compare by numeric item ID, not title. Relative
    /// dates resolve in the user's time zone: `$today`, `$today-30d`, `$today+7d`,
    /// `$this_week`, `$this_week+1w`, `$this_month`, `$this_year`
    #[serde(default)]
    pub value: serde_json::Value,
    /// How an array `value` combines: any (default, at least one matches), all
    /// (every value matches), or none (no value matches). List attributes match
    /// when any of their elements matches
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

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SortParam {
    /// `title`, `created`, `updated`, or the key of a scalar attribute (for example
    /// `TicketID`, `Priority`). Items without the attribute sort last.
    pub key: String,
    /// `asc` (default) or `desc`
    pub direction: Option<String>,
}

impl From<SortParam> for ItemSortDto {
    fn from(p: SortParam) -> Self {
        ItemSortDto {
            key: p.key,
            direction: p.direction.unwrap_or_default(),
        }
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FilterItemsParams {
    /// Include archived items (default false).
    #[serde(default)]
    pub include_archived: bool,
    /// Attribute filters. All filters are ANDed together. May be empty when `types`
    /// or `under` is given.
    #[serde(default)]
    pub filters: Vec<AttributeFilterParam>,
    /// OR groups of filters: an item matches when every filter in at least one
    /// group matches. ANDed with everything else. For example Date is today OR
    /// Week is this week: [[{"key":"Date","op":"eq","value":"$today"}],
    /// [{"key":"Week","op":"eq","value":"$this_week"}]]
    #[serde(default)]
    pub any_of: Vec<Vec<AttributeFilterParam>>,
    /// Item ID or exact title. Only return items this item links to with a
    /// `[[wikilink]]` in its content
    pub linked_from: Option<String>,
    /// Item type names the item must have (for example ["Ticket"])
    pub types: Option<Vec<String>>,
    /// `any` (default): has at least one of `types`; `all`: has every one
    pub type_mode: Option<String>,
    /// Item ID or exact title. Only return descendants of this item, following
    /// `Parent` links through any depth
    pub under: Option<String>,
    /// Sort order (default: item id)
    pub sort: Option<SortParam>,
    /// Max results (default 50, max 100)
    pub limit: Option<i64>,
    /// Offset for pagination (default 0)
    pub offset: Option<i64>,
    /// Output detail: "meta", "summary" (default), or "full"
    #[serde(default)]
    pub detail: Detail,
    /// Return only these attributes per item (plus `item_id`, `title`, `types`),
    /// overriding `detail`. For example ["TicketID", "Status"]
    pub fields: Option<Vec<String>>,
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
    /// Include archived items (default false).
    #[serde(default)]
    pub include_archived: bool,
    /// Item ID or exact title
    pub item: String,
    /// Which set of linked items to return
    pub direction: LinkDirection,
    /// Output detail: "meta", "summary" (default), or "full"
    #[serde(default)]
    pub detail: Detail,
    /// Return only these attributes per item (plus `item_id`, `title`, `types`),
    /// overriding `detail`. For example ["TicketID", "Status"]
    pub fields: Option<Vec<String>>,
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
    /// Attributes as a JSON object (e.g. {"Status": "To Do", "Date": "2026-07-03"})
    pub attributes: Option<HashMap<String, serde_json::Value>>,
    /// Type names to assign at creation
    pub types: Option<Vec<String>>,
    /// Relationship links to other items
    pub links: Option<Vec<LinkParam>>,
    /// Parent item (ID or title). Added to the "Parent" list attribute (the
    /// wiki's parent/child hierarchy), merged with any "Parent" in `attributes`
    pub parent: Option<String>,
    /// Return the complete created item instead of the compact result (default false)
    pub full: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateItemParams {
    /// Item ID or exact title
    pub item: String,
    /// New title (optional)
    pub title: Option<String>,
    /// New content body (optional). Replaces the whole body; prefer
    /// `edit_item` or `replace_section` for part of it
    pub content: Option<String>,
    /// Refuse the write if the content changed since this `version` (from
    /// `get_item`, `read_item`, or a previous write)
    pub expected_version: Option<String>,
    /// Return the complete updated item instead of the compact result (default false)
    pub full: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SetStatusParams {
    /// Item IDs or exact titles (max 50)
    pub items: Vec<String>,
    /// New Status value, e.g. "Working", "Complete", "Hold"
    pub status: String,
    /// Comment added to every item. Required when status is Hold, Blocked, or Rejected
    pub comment: Option<String>,
}

const MAX_STATUS_BATCH: usize = 50;

/// Statuses the Status Funnel requires a recorded reason for.
fn status_needs_comment(status: &str) -> bool {
    ["hold", "blocked", "rejected"].contains(&status.trim().to_lowercase().as_str())
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AppendToItemParams {
    /// Item ID or exact title
    pub item: String,
    /// Text to append as a new paragraph at the end of the item's content
    pub text: String,
    /// Refuse the write if the content changed since this `version`
    pub expected_version: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GrepItemsParams {
    /// Include archived items (default false).
    #[serde(default)]
    pub include_archived: bool,
    /// Text to find in item content, case-insensitive. Literal unless `regex` is true
    pub pattern: String,
    /// Treat `pattern` as a case-insensitive regular expression (default false)
    pub regex: Option<bool>,
    /// Item ID or exact title. Only search descendants of this item, following
    /// `Parent` links through any depth
    pub under: Option<String>,
    /// Only search items with at least one of these type names (for example ["Ticket"])
    pub types: Option<Vec<String>>,
    /// Lines of context before and after each match (default 0, max 10)
    pub context_lines: Option<usize>,
    /// Max matching items per page (default 20, max 100)
    pub limit: Option<i64>,
    /// Matching items to skip (default 0)
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReplaceSectionParams {
    /// Item ID or exact title
    pub item: String,
    /// Heading whose section to rewrite, as its text ("Goal") or with its
    /// level ("## Goal")
    pub heading: String,
    /// New body for the section, without the heading line. It replaces
    /// everything up to the next heading of the same or higher level,
    /// including nested headings
    pub content: String,
    /// Refuse the write if the content changed since this `version`
    pub expected_version: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct EditItemParams {
    /// Item ID or exact title
    pub item: String,
    /// Exact text to replace, copied from the item (without `read_item` line
    /// number prefixes). Literal, not a regex; include enough surrounding
    /// text to make it unique
    pub old: String,
    /// Replacement text
    pub new: String,
    /// Replace every occurrence instead of requiring exactly one (default false)
    pub replace_all: Option<bool>,
    /// Refuse the write if the content changed since this `version`
    pub expected_version: Option<String>,
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

/// A targeted read or edit the content rules refused: the caller's input is
/// wrong, so report it as invalid params with the reason.
fn content_err(e: ContentError) -> McpError {
    McpError::invalid_params(e.to_string(), None)
}

fn clamp_limit(limit: Option<i64>, default: i64) -> i64 {
    limit.unwrap_or(default).clamp(1, MAX_LIMIT)
}

#[rmcp::tool_router(router = wiki_tool_router, vis = "pub")]
impl ZealotServer {
    #[rmcp::tool(
        description = "Get a specific item by ID or exact title. Defaults to full content — pass detail:\"summary\" or \"meta\" for a lighter response. Every detail level includes the content `version`; pass it as `expected_version` to a content write."
    )]
    pub async fn get_item(
        &self,
        Parameters(p): Parameters<ItemRefParam>,
    ) -> Result<CallToolResult, McpError> {
        let mut item = self.resolve_item(&p.item).await?;
        item.version = output::item_version(&item);
        Ok(match p.detail {
            Detail::Full => output::json_result(&item),
            d => {
                let mut summary = json!(output::ItemSummary::project(&item, d));
                summary["version"] = json!(item.version);
                output::json_result(&summary)
            }
        })
    }

    #[rmcp::tool(
        description = "Get the heading outline of an item's content (all Markdown headings with their levels and line numbers; `#` lines in fenced code are skipped), plus its attribute keys, content length, and content `version`. Use this to understand the structure of a long item, then `read_item` one section instead of fetching the full content."
    )]
    pub async fn get_item_outline(
        &self,
        Parameters(p): Parameters<ItemOutlineParams>,
    ) -> Result<CallToolResult, McpError> {
        let item = self.resolve_item(&p.item).await?;
        let headings: Vec<serde_json::Value> = item_content::headings(&item.content)
            .into_iter()
            .map(|h| json!({"level": h.level, "text": h.text, "line": h.line}))
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
            "version": output::item_version(&item),
            "attribute_keys": attribute_keys,
            "headings": headings,
        })))
    }

    #[rmcp::tool(
        description = "Read part of an item's content with line numbers: one heading's `section` (\"Goal\" or \"## Goal\"; it runs to the next heading of the same or higher level, so it includes nested headings) or a `lines` range (\"12\", \"12-40\", \"12-\"). With neither, returns the whole body. Each line is `<number>\\t<text>`. The reply includes `version`, `total_lines`, and `start_line`/`end_line`; pass `version` as `expected_version` when you then edit. A missing or ambiguous heading fails and names the options."
    )]
    pub async fn read_item(
        &self,
        Parameters(p): Parameters<ReadItemParams>,
    ) -> Result<CallToolResult, McpError> {
        if p.section.is_some() && p.lines.is_some() {
            return Err(McpError::invalid_params(
                "pass either section or lines, not both",
                None,
            ));
        }
        let item = self.resolve_item(&p.item).await?;
        let total_lines = item_content::split_lines(&item.content).len();
        let mut heading = None;
        let range = if let Some(section) = &p.section {
            let found = item_content::find_section(&item.content, section).map_err(content_err)?;
            heading = Some(json!({"level": found.heading.level, "text": found.heading.text}));
            found.range
        } else if let Some(lines) = &p.lines {
            item_content::parse_line_range(lines, total_lines).map_err(content_err)?
        } else {
            item_content::LineRange {
                start: 1,
                end: total_lines,
            }
        };
        let mut reply = json!({
            "item_id": item.item_id,
            "title": item.title,
            "version": output::item_version(&item),
            "total_lines": total_lines,
            "start_line": range.start,
            "end_line": range.end,
            "content": item_content::numbered_lines(&item.content, range),
        });
        if let Some(heading) = heading {
            reply["heading"] = heading;
        }
        Ok(output::json_result(&reply))
    }

    #[rmcp::tool(
        description = "Browse items without a search term. mode: \"root\" (root-level items without a parent; with `type_filter`, instead every item of that type at any depth, which is how to list all Projects, Tickets, etc.; that listing takes `sort` and reports `total`), \"recent\" (most recently modified), \"random\" (random sample), or \"most_viewed\" (most-viewed items — rows are {item_id, title, view_count} regardless of detail). Returns compact summaries by default; pass detail:\"full\" for complete content."
    )]
    pub async fn browse_items(
        &self,
        Parameters(p): Parameters<BrowseItemsParams>,
    ) -> Result<CallToolResult, McpError> {
        let shape = Shape::new(p.detail, p.fields.clone());
        let limit = clamp_limit(p.limit, DEFAULT_LIMIT);
        let offset = p.offset.unwrap_or(0).max(0);
        match p.mode {
            BrowseMode::Root => match p.type_filter {
                Some(type_name) => {
                    let query = ItemQueryDto {
                        types: vec![type_name],
                        include_archived: p.include_archived,
                        sort: p.sort.map(Into::into),
                        limit: Some(limit),
                        offset,
                        ..Default::default()
                    };
                    let page = self
                        .client
                        .query_items(&query)
                        .await
                        .map_err(|e| err_ctx("items", e))?;
                    Ok(output::query_page(page, shape.clone(), offset))
                }
                None => {
                    let items = self
                        .client
                        .list_items_with_archived(None, p.include_archived)
                        .await
                        .map_err(|e| err_ctx("items", e))?;
                    Ok(slice_item_page(items, shape.clone(), offset, limit))
                }
            },
            BrowseMode::Recent => {
                let items = self
                    .client
                    .recent_items_with_archived(limit, offset, p.include_archived)
                    .await
                    .map_err(|e| err_ctx("recent items", e))?;
                Ok(output::item_page(items, shape.clone(), offset, limit))
            }
            BrowseMode::Random => {
                let items = self
                    .client
                    .random_items_with_archived(limit as usize, p.include_archived)
                    .await
                    .map_err(|e| err_ctx("random items", e))?;
                Ok(output::item_page(items, shape.clone(), 0, limit))
            }
            BrowseMode::MostViewed => {
                let rows = self
                    .client
                    .most_viewed_with_archived(limit, p.include_archived)
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
        let shape = Shape::new(p.detail, p.fields.clone());
        let limit = clamp_limit(p.limit, SEARCH_DEFAULT_LIMIT);
        let offset = p.offset.unwrap_or(0).max(0);
        let scope = match p.scope.as_deref() {
            Some("content") => SearchScope::Content,
            Some("heading") => SearchScope::Heading,
            _ => SearchScope::Title,
        };
        let hits = self
            .client
            .search_items_with_archived(
                &p.term,
                scope,
                p.regex.unwrap_or(false),
                limit,
                offset,
                p.include_archived,
            )
            .await
            .map_err(|e| err_ctx("search", e))?;
        Ok(output::search_page(hits, shape.clone(), offset, limit))
    }

    #[rmcp::tool(
        description = "Find lines in item content, like `grep -n`, matched on the server. Returns one row per matching item: `{item_id, title, match_count, lines, truncated?}`, where `lines` are `12:matching line`, `11-context line`, and `--` between groups (up to 50 matches per item). `pattern` is case-insensitive and literal unless `regex: true`. `under` (id or title) limits the search to that item's descendants; `types` to items of those types; `context_lines` adds context (max 10). Pages over matching items with `limit` (default 20) and `offset`; continue with `next_offset`. Follow up with `read_item` on a line range, then `edit_item`."
    )]
    pub async fn grep_items(
        &self,
        Parameters(p): Parameters<GrepItemsParams>,
    ) -> Result<CallToolResult, McpError> {
        let under = match &p.under {
            Some(under) => Some(self.resolve_item_id(under).await?),
            None => None,
        };
        let query = GrepQueryDto {
            include_archived: p.include_archived,
            pattern: p.pattern,
            regex: p.regex.unwrap_or(false),
            under,
            types: p.types.unwrap_or_default(),
            context_lines: p.context_lines.unwrap_or(0),
            limit: Some(clamp_limit(p.limit, SEARCH_DEFAULT_LIMIT)),
            offset: p.offset.unwrap_or(0).max(0),
        };
        let page = self
            .client
            .grep_items(&query)
            .await
            .map_err(|e| err_ctx("grep", e))?;
        Ok(output::json_result(&page))
    }

    #[rmcp::tool(
        description = "Filter items by type, subtree, and attribute values in one call; the reply includes `total` (all matches) and pages with `offset`. `types` (with `type_mode` any/all) limits to item types, e.g. [\"Ticket\"]; `under` (id or title) returns only descendants of that item through any depth of `Parent` links; `sort` is {key, direction?} over `title`, `created`, `updated`, or a scalar attribute (items lacking it sort last). At least one of `types`, `under`, `filters`, `any_of`, or `linked_from` is required. Pass `filters` as an array of {key, op, value, list_mode?}; supported ops are eq, ne, gt, lt, gte, lte, ilike, exists, and missing. `ilike` is a case-insensitive substring match on text values (the server wraps the value in `%...%`); `exists`/`missing` test whether the attribute is set at all and need no value. The keys `created` and `updated` filter on the item's timestamps by UTC day (`YYYY-MM-DD`). Values may use relative dates resolved in the user's time zone: `$today`, `$today-30d`, `$this_week`, `$this_week+1w`, `$this_month`, `$this_year`. `any_of` is a list of filter groups ORed together (each group ANDed). `linked_from` (id or title) keeps only items that item wikilinks to. `value` may be one value or an array; `list_mode` says whether any (default), all, or none of the array values must match. List attributes match when any element matches. Item-valued attributes such as `Parent` compare by numeric item ID (`{\"key\":\"Parent\",\"op\":\"eq\",\"value\":11}`). `filters` are ANDed with each other and with `any_of`. Returns compact summaries by default; pass detail:\"full\" for complete content."
    )]
    pub async fn filter_items(
        &self,
        Parameters(p): Parameters<FilterItemsParams>,
    ) -> Result<CallToolResult, McpError> {
        let shape = Shape::new(p.detail, p.fields.clone());
        let limit = clamp_limit(p.limit, DEFAULT_LIMIT);
        let offset = p.offset.unwrap_or(0).max(0);
        let under = match &p.under {
            Some(under) => Some(self.resolve_item(under).await?.item_id),
            None => None,
        };
        let linked_from = match &p.linked_from {
            Some(source) => Some(self.resolve_item(source).await?.item_id),
            None => None,
        };
        let query = ItemQueryDto {
            include_archived: p.include_archived,
            types: p.types.unwrap_or_default(),
            type_mode: p.type_mode.unwrap_or_default(),
            under,
            filters: p.filters.into_iter().map(Into::into).collect(),
            any_of: p
                .any_of
                .into_iter()
                .map(|group| group.into_iter().map(Into::into).collect())
                .collect(),
            linked_from,
            sort: p.sort.map(Into::into),
            limit: Some(limit),
            offset,
        };
        let page = self
            .client
            .query_items(&query)
            .await
            .map_err(|e| err_ctx("filtered items", e))?;
        Ok(output::query_page(page, shape.clone(), offset))
    }

    #[rmcp::tool(
        description = "List items linked to an item. direction: \"children\" (sub-items), \"related\" (all linked items), or \"backlinks\" (items that link here). Returns compact summaries by default; pass detail:\"full\" for complete content."
    )]
    pub async fn get_linked_items(
        &self,
        Parameters(p): Parameters<LinkedItemsParams>,
    ) -> Result<CallToolResult, McpError> {
        let shape = Shape::new(p.detail, p.fields.clone());
        let limit = clamp_limit(p.limit, DEFAULT_LIMIT);
        let offset = p.offset.unwrap_or(0).max(0);
        let item = self.resolve_item(&p.item).await?;
        let items = match p.direction {
            LinkDirection::Children => {
                self.client
                    .get_children_with_archived(item.item_id, p.include_archived)
                    .await
            }
            LinkDirection::Related => {
                self.client
                    .get_related_with_archived(item.item_id, p.include_archived)
                    .await
            }
            LinkDirection::Backlinks => {
                self.client
                    .get_backlinks_with_archived(item.item_id, p.include_archived)
                    .await
            }
        }
        .map_err(|e| err_ctx(&format!("links of item #{}", item.item_id), e))?;
        Ok(slice_item_page(items, shape.clone(), offset, limit))
    }

    #[rmcp::tool(
        description = "Archive or unarchive an item without deleting or stripping any history. Set archived true/false; subtree true includes all descendants through Parent links in one atomic action. Direct reads and wikilinks keep working. Read the target first; obtain explicit authority before archiving real items."
    )]
    pub async fn set_archived(
        &self,
        Parameters(p): Parameters<ArchiveItemParams>,
    ) -> Result<CallToolResult, McpError> {
        let id = self.resolve_item_id(&p.item).await?;
        let result = self
            .client
            .set_archived(id, p.archived, p.subtree)
            .await
            .map_err(|e| err_ctx("archive item", e))?;
        Ok(output::json_result(&result))
    }

    #[rmcp::tool(
        description = "Create a new wiki item. `content` is ZealotScript/markdown. `attributes` is a JSON object of key→value pairs. `types` assigns item types at creation. `links` sets relationships to other items by ID. `parent` (ID or title) adds that item to the `Parent` list attribute, placing the new item in the wiki hierarchy; it merges with any `Parent` given in `attributes`. Returns a compact result (`item_id`, `title`, `types`, `attributes`, `content_chars`); pass `full: true` for the complete item."
    )]
    pub async fn create_item(
        &self,
        Parameters(p): Parameters<CreateItemParams>,
    ) -> Result<CallToolResult, McpError> {
        let mut attributes = p.attributes.unwrap_or_default();
        if let Some(parent) = &p.parent {
            let parent = self.resolve_item(parent).await?;
            merge_parent(&mut attributes, parent.item_id);
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
        Ok(output::write_item_result(&item, p.full.unwrap_or(false)))
    }

    #[rmcp::tool(
        description = "Update an existing item's title and/or replace its whole content. Only provided fields are changed. To change part of the body use `edit_item` or `replace_section` instead. Pass `expected_version` (from a read) to refuse the write if the content changed since. Returns `{item_id, title, version, content_chars, changed_lines}`; pass `full: true` for the complete item."
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
            expected_version: p.expected_version,
        };
        let item = self
            .client
            .update_item(&dto)
            .await
            .map_err(|e| err_ctx(&format!("item #{}", existing.item_id), e))?;
        if p.full.unwrap_or(false) {
            return Ok(output::json_result(&item));
        }
        Ok(output::json_result(&ContentWriteDto {
            item_id: item.item_id,
            title: item.title.clone(),
            version: output::item_version(&item),
            content_chars: item.content.chars().count(),
            changed_lines: item_content::changed_lines(&existing.content, &item.content),
        }))
    }

    #[rmcp::tool(
        description = "Append text as a new paragraph at the end of an item's content. The append runs on the server in one write, so it never overwrites a concurrent edit; `expected_version` additionally refuses it if the item changed since you read it. Replies `{item_id, title, version, content_chars, changed_lines}`."
    )]
    pub async fn append_to_item(
        &self,
        Parameters(p): Parameters<AppendToItemParams>,
    ) -> Result<CallToolResult, McpError> {
        let edit = ContentEdit::Append { text: p.text };
        self.edit_content(&p.item, edit, p.expected_version).await
    }

    #[rmcp::tool(
        description = "Replace exact text in an item's content on the server, without resending the body. `old` must match exactly once (copy it from `read_item` without the line-number prefix, with enough context to be unique) unless `replace_all` is true; it is literal text, never a regex. Pass the `version` you read as `expected_version` so the edit is refused if the item changed since. Fails, naming the reason, when `old` is not found, matches more than once, or the version is stale. Replies `{item_id, title, version, content_chars, changed_lines: {start, removed, added}}`."
    )]
    pub async fn edit_item(
        &self,
        Parameters(p): Parameters<EditItemParams>,
    ) -> Result<CallToolResult, McpError> {
        let edit = ContentEdit::Replace {
            old: p.old,
            new: p.new,
            replace_all: p.replace_all.unwrap_or(false),
        };
        self.edit_content(&p.item, edit, p.expected_version).await
    }

    #[rmcp::tool(
        description = "Rewrite one section of an item's content on the server. `heading` is the heading text (\"Goal\") or text with level (\"## Goal\"); `content` is the new section body without the heading line, and replaces everything up to the next heading of the same or higher level (nested headings included). The heading line is kept. Fails if the heading is missing or ambiguous, or if `expected_version` is stale. Replies `{item_id, title, version, content_chars, changed_lines}`."
    )]
    pub async fn replace_section(
        &self,
        Parameters(p): Parameters<ReplaceSectionParams>,
    ) -> Result<CallToolResult, McpError> {
        let edit = ContentEdit::ReplaceSection {
            heading: p.heading,
            content: p.content,
        };
        self.edit_content(&p.item, edit, p.expected_version).await
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
        description = "Set Status on several items in one call, optionally adding the same comment to each (a comment is required for Hold, Blocked, and Rejected). Items are processed independently: the reply has one compact row per item ({item_id, title, status, comment_id?} or {item, error}) so partial failures are visible."
    )]
    pub async fn set_status(
        &self,
        Parameters(p): Parameters<SetStatusParams>,
    ) -> Result<CallToolResult, McpError> {
        if p.items.is_empty() || p.items.len() > MAX_STATUS_BATCH {
            return Err(McpError::invalid_params(
                format!("items must list 1 to {MAX_STATUS_BATCH} items"),
                None,
            ));
        }
        let comment = p
            .comment
            .as_deref()
            .map(str::trim)
            .filter(|c| !c.is_empty());
        if comment.is_none() && status_needs_comment(&p.status) {
            return Err(McpError::invalid_params(
                format!(
                    "a comment giving the reason is required when setting Status to {}",
                    p.status
                ),
                None,
            ));
        }

        let mut rows = Vec::with_capacity(p.items.len());
        let mut failed = 0;
        for reference in &p.items {
            match self.set_one_status(reference, &p.status, comment).await {
                Ok(row) => rows.push(row),
                Err(e) => {
                    failed += 1;
                    rows.push(json!({"item": reference, "error": e.message}));
                }
            }
        }
        Ok(output::json_result(&json!({
            "updated": rows.len() - failed,
            "failed": failed,
            "items": rows,
        })))
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

impl ZealotServer {
    /// Send one targeted content edit to the server and return its compact reply.
    async fn edit_content(
        &self,
        reference: &str,
        edit: ContentEdit,
        expected_version: Option<String>,
    ) -> Result<CallToolResult, McpError> {
        let item = self.resolve_item_id(reference).await?;
        let dto = ContentEditDto {
            edit,
            expected_version,
        };
        let written = self
            .client
            .edit_item_content(item, &dto)
            .await
            .map_err(|e| err_ctx(&format!("item #{item}"), e))?;
        Ok(output::json_result(&written))
    }

    async fn set_one_status(
        &self,
        reference: &str,
        status: &str,
        comment: Option<&str>,
    ) -> Result<serde_json::Value, McpError> {
        let item = self.resolve_item(reference).await?;
        let attrs = HashMap::from([("Status".to_string(), json!(status))]);
        self.client
            .set_item_attributes(item.item_id, &attrs)
            .await
            .map_err(|e| err_ctx(&format!("item #{}", item.item_id), e))?;
        let mut row = json!({"item_id": item.item_id, "title": item.title, "status": status});
        if let Some(content) = comment {
            let dto = AddCommentDto {
                item_id: item.item_id,
                timestamp: now_timestamp(),
                content: content.to_string(),
            };
            let added = self.client.add_comment(&dto).await.map_err(|e| {
                let e = err_ctx(&format!("comment on item #{}", item.item_id), e);
                McpError::internal_error(
                    format!(
                        "Status was set to {status} but the comment failed: {}",
                        e.message
                    ),
                    None,
                )
            })?;
            row["comment_id"] = json!(added.comment_id);
        }
        Ok(row)
    }
}

/// Add `parent_id` to the `Parent` attribute, which is `list(item)` on the
/// server. Any `Parent` already in `attributes` (a single value or an array) is
/// kept, in order, and the resolved parent is appended unless already present.
fn merge_parent(attributes: &mut HashMap<String, serde_json::Value>, parent_id: i64) {
    let mut parents = match attributes.remove("Parent") {
        Some(serde_json::Value::Array(values)) => values,
        Some(serde_json::Value::Null) | None => Vec::new(),
        Some(value) => vec![value],
    };
    let parent = json!(parent_id);
    if !parents.contains(&parent) {
        parents.push(parent);
    }
    attributes.insert("Parent".to_string(), serde_json::Value::Array(parents));
}

/// `list_items` (root/type-filtered) is unbounded server-side; slice + project client-side.
fn slice_item_page(items: Vec<ItemDto>, shape: Shape, offset: i64, limit: i64) -> CallToolResult {
    let start = offset.max(0) as usize;
    let end = start.saturating_add(limit.max(0) as usize);
    let sliced: Vec<ItemDto> = items.into_iter().skip(start).take(end - start).collect();
    output::item_page(sliced, shape, offset, limit)
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ArchiveItemParams {
    /// Item ID or exact title
    pub item: String,
    /// true archives; false unarchives
    pub archived: bool,
    /// Apply to this item and all descendants (default false)
    #[serde(default)]
    pub subtree: bool,
}

//! Shared output shaping: detail levels, item projections, and compact JSON
//! serialization used by every read tool.

use rmcp::model::{CallToolResult, Content};
use serde::{Deserialize, Serialize};
use zealot_domain::{
    comment::CommentDto,
    item::{ItemDto, ItemQueryPageDto, SearchResultDto},
};

pub const PREVIEW_CHARS: usize = 120;
pub const DEFAULT_LIMIT: i64 = 50;

/// Output detail level for item-returning tools.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Detail {
    /// id, title, types only
    Meta,
    /// meta + attributes + a short content preview (default for list/search tools)
    #[default]
    Summary,
    /// the complete item, including full content and links
    Full,
}

/// How list tools shape each item: a detail level, or only the named attributes (`fields`).
#[derive(Debug, Clone)]
pub enum Shape {
    Detail(Detail),
    Fields(Vec<String>),
}

impl Shape {
    /// `fields`, when given, takes precedence over `detail`.
    pub fn new(detail: Detail, fields: Option<Vec<String>>) -> Self {
        match fields {
            Some(fields) => Shape::Fields(fields),
            None => Shape::Detail(detail),
        }
    }
}

impl From<Detail> for Shape {
    fn from(detail: Detail) -> Self {
        Shape::Detail(detail)
    }
}

/// `item_id`, `title`, `types`, and only the requested attributes.
#[derive(Debug, Serialize)]
pub struct FieldRow {
    pub archived: bool,
    pub item_id: i64,
    pub title: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub types: Vec<String>,
    #[serde(skip_serializing_if = "serde_json::Map::is_empty")]
    pub attributes: serde_json::Map<String, serde_json::Value>,
}

impl FieldRow {
    pub fn project(item: &ItemDto, fields: &[String]) -> Self {
        let attributes = item
            .attributes
            .as_object()
            .map(|all| {
                fields
                    .iter()
                    .filter_map(|f| all.get(f).map(|v| (f.clone(), v.clone())))
                    .collect()
            })
            .unwrap_or_default();
        Self {
            archived: item.archived,
            item_id: item.item_id,
            title: item.title.clone(),
            types: item.types.iter().map(|t| t.name.clone()).collect(),
            attributes,
        }
    }
}

/// Whitespace-collapse and truncate `content` to at most `max` chars (char-safe).
pub fn preview(content: &str, max: usize) -> Option<String> {
    if content.is_empty() {
        return None;
    }
    let collapsed = content.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.is_empty() {
        return None;
    }
    if collapsed.chars().count() <= max {
        return Some(collapsed);
    }
    let truncated: String = collapsed.chars().take(max).collect();
    Some(format!("{truncated}…"))
}

#[derive(Debug, Serialize)]
pub struct ItemSummary {
    pub archived: bool,
    pub item_id: i64,
    pub title: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub types: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<String>,
    pub content_chars: usize,
}

impl ItemSummary {
    pub fn project(item: &ItemDto, detail: Detail) -> Self {
        Self {
            archived: item.archived,
            item_id: item.item_id,
            title: item.title.clone(),
            types: item.types.iter().map(|t| t.name.clone()).collect(),
            attributes: (detail != Detail::Meta).then(|| item.attributes.clone()),
            preview: (detail != Detail::Meta)
                .then(|| preview(&item.content, PREVIEW_CHARS))
                .flatten(),
            content_chars: item.content.chars().count(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SearchHitSummary {
    #[serde(flatten)]
    pub item: ItemSummary,
    pub match_scope: zealot_domain::item::SearchScope,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
}

impl SearchHitSummary {
    pub fn project(hit: &SearchResultDto, detail: Detail) -> Self {
        Self {
            item: ItemSummary::project(&hit.item, detail),
            match_scope: hit.match_scope.clone(),
            snippet: hit.snippet.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SearchFieldHit {
    #[serde(flatten)]
    pub row: FieldRow,
    pub match_scope: zealot_domain::item::SearchScope,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
}

/// The item's content version, computed locally when the server predates
/// `ItemDto::version` so reads always carry one.
pub fn item_version(item: &ItemDto) -> String {
    if item.version.is_empty() {
        zealot_domain::item_content::content_version(&item.content)
    } else {
        item.version.clone()
    }
}

/// Compact JSON in a single text content block.
pub fn json_result<T: Serialize>(value: &T) -> CallToolResult {
    CallToolResult::success(vec![Content::text(
        serde_json::to_string(value).unwrap_or_default(),
    )])
}

/// Result of a create/update: compact by default, the full item when `full` is set.
pub fn write_item_result(item: &ItemDto, full: bool) -> CallToolResult {
    if full {
        return json_result(item);
    }
    json_result(&serde_json::json!({
        "item_id": item.item_id,
        "title": item.title,
        "types": item.types.iter().map(|t| t.name.as_str()).collect::<Vec<_>>(),
        "attributes": item.attributes,
        "content_chars": item.content.chars().count(),
    }))
}

/// Result of adding a comment: compact by default (no parent item body), the full comment when `full` is set.
pub fn write_comment_result(comment: &CommentDto, full: bool) -> CallToolResult {
    if full {
        return json_result(comment);
    }
    json_result(&serde_json::json!({
        "comment_id": comment.comment_id,
        "item_id": comment.item.item_id,
        "timestamp": comment.timestamp,
    }))
}

#[derive(Debug, Serialize)]
pub struct Page<T: Serialize> {
    pub count: usize,
    /// Number of all matching items, when the backing query reports it
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_offset: Option<i64>,
    pub items: Vec<T>,
}

impl<T: Serialize> Page<T> {
    pub fn new(items: Vec<T>, offset: i64, limit: i64) -> Self {
        let count = items.len();
        let next_offset = (count as i64 == limit).then_some(offset + limit);
        Self {
            count,
            total: None,
            next_offset,
            items,
        }
    }

    /// A page whose `total` is known: more remain while `offset + count < total`.
    pub fn with_total(items: Vec<T>, offset: i64, total: i64) -> Self {
        let count = items.len();
        let next_offset = (offset + (count as i64) < total).then_some(offset + count as i64);
        Self {
            count,
            total: Some(total),
            next_offset,
            items,
        }
    }
}

/// Render one page of a server-side query with its total match count.
pub fn query_page(page: ItemQueryPageDto, shape: impl Into<Shape>, offset: i64) -> CallToolResult {
    let total = page.total;
    match shape.into() {
        Shape::Detail(Detail::Full) => json_result(&Page::with_total(page.items, offset, total)),
        Shape::Detail(detail) => {
            let projected: Vec<ItemSummary> = page
                .items
                .iter()
                .map(|i| ItemSummary::project(i, detail))
                .collect();
            json_result(&Page::with_total(projected, offset, total))
        }
        Shape::Fields(fields) => {
            let rows: Vec<FieldRow> = page
                .items
                .iter()
                .map(|i| FieldRow::project(i, &fields))
                .collect();
            json_result(&Page::with_total(rows, offset, total))
        }
    }
}

/// Render a page of items in the requested shape. `Full` serializes the raw DTOs.
pub fn item_page(
    items: Vec<ItemDto>,
    shape: impl Into<Shape>,
    offset: i64,
    limit: i64,
) -> CallToolResult {
    match shape.into() {
        Shape::Detail(Detail::Full) => json_result(&Page::new(items, offset, limit)),
        Shape::Detail(detail) => {
            let projected: Vec<ItemSummary> = items
                .iter()
                .map(|i| ItemSummary::project(i, detail))
                .collect();
            json_result(&Page::new(projected, offset, limit))
        }
        Shape::Fields(fields) => {
            let rows: Vec<FieldRow> = items
                .iter()
                .map(|i| FieldRow::project(i, &fields))
                .collect();
            json_result(&Page::new(rows, offset, limit))
        }
    }
}

/// Render a page of search hits in the requested shape.
pub fn search_page(
    hits: Vec<SearchResultDto>,
    shape: impl Into<Shape>,
    offset: i64,
    limit: i64,
) -> CallToolResult {
    match shape.into() {
        Shape::Detail(Detail::Full) => json_result(&Page::new(hits, offset, limit)),
        Shape::Detail(detail) => {
            let projected: Vec<SearchHitSummary> = hits
                .iter()
                .map(|h| SearchHitSummary::project(h, detail))
                .collect();
            json_result(&Page::new(projected, offset, limit))
        }
        Shape::Fields(fields) => {
            let rows: Vec<SearchFieldHit> = hits
                .iter()
                .map(|h| SearchFieldHit {
                    row: FieldRow::project(&h.item, &fields),
                    match_scope: h.match_scope.clone(),
                    snippet: h.snippet.clone(),
                })
                .collect();
            json_result(&Page::new(rows, offset, limit))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(content: &str) -> ItemDto {
        ItemDto {
            archived: false,
            item_id: 1,
            title: "Test".into(),
            content: content.into(),
            attributes: serde_json::json!({"Status": "Open"}),
            types: vec![],
            links: vec![],
            version: String::new(),
        }
    }

    #[test]
    fn preview_collapses_whitespace() {
        assert_eq!(
            preview("hello   \n\n  world", 100),
            Some("hello world".to_string())
        );
    }

    #[test]
    fn preview_truncates_char_safely() {
        let content = "a".repeat(200);
        let p = preview(&content, 10).unwrap();
        assert_eq!(p.chars().count(), 11); // 10 chars + ellipsis
        assert!(p.ends_with('…'));
    }

    #[test]
    fn preview_empty_is_none() {
        assert_eq!(preview("", 10), None);
        assert_eq!(preview("   ", 10), None);
    }

    #[test]
    fn meta_detail_omits_attrs_and_preview() {
        let s = ItemSummary::project(&item("some content"), Detail::Meta);
        assert!(s.attributes.is_none());
        assert!(s.preview.is_none());
        assert_eq!(s.content_chars, 12);
    }

    #[test]
    fn summary_detail_includes_attrs_and_preview() {
        let s = ItemSummary::project(&item("some content"), Detail::Summary);
        assert!(s.attributes.is_some());
        assert_eq!(s.preview.as_deref(), Some("some content"));
    }

    #[test]
    fn next_offset_present_iff_full_page() {
        let full: Page<i32> = Page::new(vec![1, 2, 3], 0, 3);
        assert_eq!(full.next_offset, Some(3));
        let short: Page<i32> = Page::new(vec![1, 2], 0, 3);
        assert_eq!(short.next_offset, None);
    }
}

/// Project items to JSON in the requested shape (the same shapes as `query_page`).
fn project_items(items: &[ItemDto], shape: &Shape) -> serde_json::Value {
    let value = match shape {
        Shape::Detail(Detail::Full) => serde_json::to_value(items),
        Shape::Detail(detail) => serde_json::to_value(
            items
                .iter()
                .map(|i| ItemSummary::project(i, *detail))
                .collect::<Vec<_>>(),
        ),
        Shape::Fields(fields) => serde_json::to_value(
            items
                .iter()
                .map(|i| FieldRow::project(i, fields))
                .collect::<Vec<_>>(),
        ),
    };
    value.unwrap_or_default()
}

/// Render a Saved View run: paging like `query_page`, plus the view's groups and metadata.
pub fn view_result(
    result: zealot_domain::saved_view::ViewResultDto,
    shape: Shape,
) -> CallToolResult {
    let count = result.items.len() as i64;
    let next_offset = (result.offset + count < result.total).then_some(result.offset + count);
    let mut out = serde_json::json!({
        "view_id": result.view_id,
        "title": result.title,
        "total": result.total,
        "count": count,
        "items": project_items(&result.items, &shape),
    });
    let obj = out.as_object_mut().expect("object literal");
    if let Some(next_offset) = next_offset {
        obj.insert("next_offset".into(), next_offset.into());
    }
    if let Some(group_by) = result.group_by {
        obj.insert("group_by".into(), group_by.into());
        obj.insert(
            "groups".into(),
            serde_json::to_value(result.groups).unwrap_or_default(),
        );
    }
    if !result.warnings.is_empty() {
        obj.insert("warnings".into(), result.warnings.into());
    }
    json_result(&out)
}

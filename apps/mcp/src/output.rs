//! Shared output shaping: detail levels, item projections, and compact JSON
//! serialization used by every read tool.

use rmcp::model::{CallToolResult, Content};
use serde::{Deserialize, Serialize};
use zealot_domain::item::{ItemDto, SearchResultDto};

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

/// Compact JSON in a single text content block.
pub fn json_result<T: Serialize>(value: &T) -> CallToolResult {
    CallToolResult::success(vec![Content::text(
        serde_json::to_string(value).unwrap_or_default(),
    )])
}

#[derive(Debug, Serialize)]
pub struct Page<T: Serialize> {
    pub count: usize,
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
            next_offset,
            items,
        }
    }
}

/// Render a page of items at the requested detail. `Full` serializes the raw DTOs.
pub fn item_page(items: Vec<ItemDto>, detail: Detail, offset: i64, limit: i64) -> CallToolResult {
    match detail {
        Detail::Full => json_result(&Page::new(items, offset, limit)),
        _ => {
            let projected: Vec<ItemSummary> = items
                .iter()
                .map(|i| ItemSummary::project(i, detail))
                .collect();
            json_result(&Page::new(projected, offset, limit))
        }
    }
}

/// Render a page of search hits at the requested detail.
pub fn search_page(
    hits: Vec<SearchResultDto>,
    detail: Detail,
    offset: i64,
    limit: i64,
) -> CallToolResult {
    match detail {
        Detail::Full => json_result(&Page::new(hits, offset, limit)),
        _ => {
            let projected: Vec<SearchHitSummary> = hits
                .iter()
                .map(|h| SearchHitSummary::project(h, detail))
                .collect();
            json_result(&Page::new(projected, offset, limit))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(content: &str) -> ItemDto {
        ItemDto {
            item_id: 1,
            title: "Test".into(),
            content: content.into(),
            attributes: serde_json::json!({"Status": "Open"}),
            types: vec![],
            links: vec![],
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

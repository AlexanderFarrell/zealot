//! Saved Views: items of type `Saved View` whose content holds a re-runnable item query.
//!
//! The definition lives in the item content as one fenced block tagged `zealot-view`, so it is
//! versioned, diffable, exported and edited like any other content:
//!
//! ````text
//! ```zealot-view
//! {
//!   "query": { "filters": [{ "key": "Executor", "op": "eq", "value": "LLM" }] },
//!   "group_by": "Parent",
//!   "columns": ["Status", "Priority"]
//! }
//! ```
//! ````
//!
//! Prose around the block documents the view. `query` is an [`ItemQueryDto`], so everything
//! `POST /item/query` and MCP `filter_items` accept works here, including relative date tokens.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::item::{ItemDto, ItemQueryDto};

/// Name of the system item type that marks an item as a Saved View.
pub const SAVED_VIEW_TYPE: &str = "Saved View";

/// Info string of the fenced block that holds a view definition.
pub const VIEW_FENCE_LANG: &str = "zealot-view";

/// Keys `ItemQueryDto` understands; anything else in `query` is reported as a warning,
/// because serde would otherwise ignore a typo such as `filter` silently.
const QUERY_KEYS: &[&str] = &[
    "types",
    "include_archived",
    "type_mode",
    "under",
    "filters",
    "any_of",
    "linked_from",
    "sort",
    "limit",
    "offset",
];

/// A parsed view definition.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavedViewDefinition {
    /// The item query the view runs
    pub query: ItemQueryDto,
    /// Attribute key to group results by (each list value forms its own group)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_by: Option<String>,
    /// Attribute keys a client should show per item; informational for API callers
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub columns: Vec<String>,
}

/// A definition plus the non-fatal problems found while parsing it.
#[derive(Debug, Clone)]
pub struct ParsedView {
    pub definition: SavedViewDefinition,
    pub warnings: Vec<String>,
}

/// Why a view definition could not be read.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum SavedViewError {
    #[error("no ```zealot-view block found in the item content")]
    MissingBlock,
    #[error("more than one ```zealot-view block found; a Saved View holds exactly one")]
    MultipleBlocks,
    #[error("the ```zealot-view block is not closed")]
    UnclosedBlock,
    #[error("invalid view definition: {0}")]
    Invalid(String),
}

/// Find the single `zealot-view` fenced block in `content` and return its body.
pub fn extract_definition_source(content: &str) -> Result<String, SavedViewError> {
    let mut found: Option<String> = None;
    let mut lines = content.lines();
    while let Some(line) = lines.next() {
        let Some((fence, info)) = opening_fence(line) else {
            continue;
        };
        let mut body = Vec::new();
        let mut closed = false;
        for inner in lines.by_ref() {
            if is_closing_fence(inner, &fence) {
                closed = true;
                break;
            }
            body.push(inner);
        }
        if info.split_whitespace().next() != Some(VIEW_FENCE_LANG) {
            // Some other code block: skip it whole so a `zealot-view` example inside it is ignored.
            continue;
        }
        if !closed {
            return Err(SavedViewError::UnclosedBlock);
        }
        if found.is_some() {
            return Err(SavedViewError::MultipleBlocks);
        }
        found = Some(body.join("\n"));
    }
    found.ok_or(SavedViewError::MissingBlock)
}

/// The fence marker (``` or ~~~, possibly longer) and info string of an opening fence line.
fn opening_fence(line: &str) -> Option<(String, &str)> {
    let trimmed = line.trim_start();
    let marker = trimmed.chars().next().filter(|c| *c == '`' || *c == '~')?;
    let len = trimmed.chars().take_while(|c| *c == marker).count();
    if len < 3 {
        return None;
    }
    let fence: String = std::iter::repeat_n(marker, len).collect();
    Some((fence, trimmed[len..].trim()))
}

/// A closing fence uses the same marker, at least as long, with nothing after it.
fn is_closing_fence(line: &str, fence: &str) -> bool {
    let trimmed = line.trim();
    let marker = fence.chars().next().unwrap_or('`');
    trimmed.len() >= fence.len() && trimmed.chars().all(|c| c == marker)
}

/// Parse a definition from JSON text.
pub fn parse_definition(source: &str) -> Result<ParsedView, SavedViewError> {
    let value: Value =
        serde_json::from_str(source).map_err(|e| SavedViewError::Invalid(e.to_string()))?;
    parse_definition_value(&value)
}

/// Parse a definition from a JSON value, collecting warnings for ignored query keys.
pub fn parse_definition_value(value: &Value) -> Result<ParsedView, SavedViewError> {
    let definition: SavedViewDefinition = serde_json::from_value(value.clone())
        .map_err(|e| SavedViewError::Invalid(e.to_string()))?;
    let mut warnings = Vec::new();
    if let Some(query) = value.get("query").and_then(Value::as_object) {
        for key in query.keys() {
            if !QUERY_KEYS.contains(&key.as_str()) {
                warnings.push(format!(
                    "query key '{key}' is not recognised and was ignored"
                ));
            }
        }
    }
    Ok(ParsedView {
        definition,
        warnings,
    })
}

/// Read and parse the definition held in an item's content.
pub fn parse_content(content: &str) -> Result<ParsedView, SavedViewError> {
    parse_definition(&extract_definition_source(content)?)
}

/// Paging for a view run. `limit` defaults to the definition's limit, then 50; capped at 100.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunViewDto {
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub offset: Option<i64>,
}

/// Body of `POST /view/validate`: a definition that is not saved anywhere yet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateViewDto {
    pub definition: Value,
}

/// One group of a grouped view run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewGroupDto {
    /// The attribute value shared by the group (`null` for items without it)
    pub key: Value,
    /// Readable key: the item title for item-valued attributes, `(none)` for null
    pub label: String,
    /// Items in this group, in result order
    pub item_ids: Vec<i64>,
}

/// The result of running a view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewResultDto {
    pub view_id: i64,
    pub title: String,
    pub total: i64,
    pub offset: i64,
    pub items: Vec<ItemDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_by: Option<String>,
    /// Groups over the returned page only; empty when the view has no `group_by`
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<ViewGroupDto>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub columns: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

/// How a view is evaluated: the definition, the query after token resolution, and the cost.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewInspectDto {
    /// `None` when inspecting an unsaved definition (`validate`)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub view_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub definition: SavedViewDefinition,
    /// The query exactly as executed: relative dates replaced by concrete values
    pub resolved_query: ItemQueryDto,
    /// The date tokens resolved against, in `time_zone`
    pub today: String,
    pub time_zone: String,
    pub total: i64,
    pub elapsed_ms: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const VIEW: &str = "# AI Queue\n\nWork an agent can pick up.\n\n```zealot-view\n{\"query\": {\"filters\": [{\"key\": \"Executor\", \"op\": \"eq\", \"value\": \"LLM\"}]}, \"group_by\": \"Parent\"}\n```\n";

    #[test]
    fn parses_the_block_and_ignores_prose() {
        let parsed = parse_content(VIEW).unwrap();
        assert_eq!(parsed.definition.query.filters.len(), 1);
        assert_eq!(parsed.definition.group_by.as_deref(), Some("Parent"));
        assert!(parsed.warnings.is_empty());
    }

    #[test]
    fn skips_other_code_blocks_including_examples_inside_them() {
        let content = format!("````markdown\n```zealot-view\n{{}}\n```\n````\n\n{VIEW}");
        assert!(parse_content(&content).is_ok());
    }

    #[test]
    fn reports_missing_multiple_and_unclosed_blocks() {
        assert_eq!(
            parse_content("no view").unwrap_err(),
            SavedViewError::MissingBlock
        );
        assert_eq!(
            parse_content(&format!("{VIEW}\n{VIEW}")).unwrap_err(),
            SavedViewError::MultipleBlocks
        );
        assert_eq!(
            parse_content("```zealot-view\n{}").unwrap_err(),
            SavedViewError::UnclosedBlock
        );
    }

    #[test]
    fn rejects_bad_json_and_unknown_top_level_keys() {
        assert!(matches!(
            parse_definition("{"),
            Err(SavedViewError::Invalid(_))
        ));
        assert!(matches!(
            parse_definition("{\"query\": {}, \"group\": \"Parent\"}"),
            Err(SavedViewError::Invalid(_))
        ));
        assert!(matches!(
            parse_definition("{}"),
            Err(SavedViewError::Invalid(_))
        ));
    }

    #[test]
    fn warns_about_unknown_query_keys() {
        let parsed =
            parse_definition("{\"query\": {\"types\": [\"Ticket\"], \"filter\": []}}").unwrap();
        assert_eq!(parsed.warnings.len(), 1);
        assert!(parsed.warnings[0].contains("'filter'"));
    }

    #[test]
    fn tilde_fences_work() {
        let content = "~~~zealot-view\n{\"query\": {\"types\": [\"Ticket\"]}}\n~~~";
        assert_eq!(
            parse_content(content).unwrap().definition.query.types,
            ["Ticket"]
        );
    }
}

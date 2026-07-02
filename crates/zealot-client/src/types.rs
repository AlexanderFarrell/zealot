//! Client-side response types for endpoints whose server-side shapes are not
//! exported from `zealot-domain` (handler-local structs and ad-hoc JSON).

use serde::{Deserialize, Serialize};
use serde_json::Value;
use zealot_domain::media::FileStatDto;

/// Result of `POST /rule/{id}/run` (mirrors zealot-app's `RuleRunResult`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleRunResultDto {
    pub rule_id: i64,
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

/// Attribute kind as returned by `GET /attribute/` (the handler builds this
/// JSON by hand; `base_type` is a lowercase string like "text" or "list").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeKindDto {
    pub kind_id: i64,
    pub key: String,
    pub description: String,
    pub is_system: bool,
    pub base_type: String,
    #[serde(default)]
    pub config: Value,
}

/// Result of `POST /item/rebuild-links`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebuildLinksResultDto {
    pub rebuilt: i64,
    pub wiki_rebuilt: i64,
}

/// Result row of `GET /analysis/most-viewed`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MostViewedItemDto {
    pub item_id: i64,
    pub title: String,
    pub view_count: i64,
}

/// `GET /media/{path}` returns either a directory listing (JSON) or file bytes.
#[derive(Debug, Clone)]
pub enum MediaEntry {
    Directory(Vec<FileStatDto>),
    File(crate::http::Download),
}

/// Wire shape of a directory listing from `GET /media/…`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryResponse {
    pub files: Vec<FileStatDto>,
}

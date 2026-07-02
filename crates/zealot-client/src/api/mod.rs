//! Typed endpoint wrappers over the Zealot REST API, grouped by area.
//! Each module adds methods to [`crate::ZealotClient`] via `impl` blocks.
//! The backend handlers in `crates/zealot-api/src/http/` are the source of
//! truth for paths, query params, and response shapes.

pub mod account;
pub mod attributes;
pub mod auth;
pub mod comments;
pub mod item_types;
pub mod items;
pub mod media;
pub mod planner;
pub mod repeats;
pub mod rules;
pub mod time_blocks;

/// Percent-encode a single path segment (titles, type names, keys).
pub(crate) fn seg(value: &str) -> String {
    urlencoding::encode(value).into_owned()
}

/// Percent-encode a media path, preserving `/` separators.
pub(crate) fn media_path(path: &str) -> String {
    path.trim_matches('/')
        .split('/')
        .map(|part| urlencoding::encode(part).into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

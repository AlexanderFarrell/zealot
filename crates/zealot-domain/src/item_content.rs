//! Line-oriented operations on item content: the content version used for
//! optimistic concurrency, heading sections, line ranges, and exact-text edits.
//!
//! These are pure functions so the API (which applies edits) and the MCP
//! (which reads sections) agree on what a version, a line, or a section is.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Hex characters kept from the SHA-256 digest. 64 bits is ample to detect a
/// changed body while staying short enough for agents to echo back.
const VERSION_HEX_CHARS: usize = 16;

/// The version of an item's content: a short SHA-256 hash of the body.
///
/// It changes whenever the content changes, and only then, so a writer can
/// send the version it read and have the write refused if someone else edited
/// the body in between. Unlike `updated_at` it has no one-second blind spot.
pub fn content_version(content: &str) -> String {
    let digest = Sha256::digest(content.as_bytes());
    let mut version = hex::encode(digest);
    version.truncate(VERSION_HEX_CHARS);
    version
}

/// Why a targeted content read or edit could not be applied.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ContentError {
    #[error("heading '{heading}' not found; headings are: {available}")]
    HeadingNotFound { heading: String, available: String },

    #[error(
        "heading '{heading}' is ambiguous: it appears on lines {lines:?}; include its level (for example '## {heading}') or read by line range"
    )]
    HeadingAmbiguous { heading: String, lines: Vec<usize> },

    #[error("heading must not be empty")]
    EmptyHeading,

    #[error("invalid line range '{range}': {reason}")]
    InvalidLineRange { range: String, reason: String },

    #[error("old text must not be empty")]
    EmptyOld,

    #[error("old text not found in the item content; re-read the item and copy the text exactly")]
    TextNotFound,

    #[error(
        "old text matches {count} times (lines {lines:?}); add surrounding text to make it unique, or set replace_all"
    )]
    TextAmbiguous { count: usize, lines: Vec<usize> },

    #[error(
        "the item changed since version {expected} (now {actual}); re-read it and retry the edit"
    )]
    VersionConflict { expected: String, actual: String },
}

/// A Markdown ATX heading (`# Title` through `###### Title`) in item content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    /// 1-based line number of the heading line
    pub line: usize,
    /// 1 to 6
    pub level: u8,
    /// Heading text without the `#` marks, trimmed
    pub text: String,
}

/// An inclusive, 1-based range of content lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineRange {
    pub start: usize,
    pub end: usize,
}

/// A heading's section: the heading line up to, not including, the next
/// heading of the same or higher level (or the end of the content).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    pub heading: Heading,
    pub range: LineRange,
}

/// Content split into lines on `\n`. A trailing newline does not start an
/// extra empty line, so `"a\nb\n"` has two lines, like a text editor shows.
pub fn split_lines(content: &str) -> Vec<&str> {
    if content.is_empty() {
        return Vec::new();
    }
    content
        .strip_suffix('\n')
        .unwrap_or(content)
        .split('\n')
        .collect()
}

/// Parse one line as an ATX heading, returning its level and text.
fn parse_heading(line: &str) -> Option<(u8, String)> {
    let trimmed = line.trim_end_matches('\r').trim_start();
    let level = trimmed.chars().take_while(|&c| c == '#').count();
    if !(1..=6).contains(&level) {
        return None;
    }
    let rest = &trimmed[level..];
    if !(rest.is_empty() || rest.starts_with(' ') || rest.starts_with('\t')) {
        return None;
    }
    let text = rest.trim().to_string();
    (!text.is_empty()).then_some((level as u8, text))
}

/// Whether a line opens or closes a fenced code block (``` or ~~~).
fn is_fence(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("```") || trimmed.starts_with("~~~")
}

/// Every heading in the content, in order, ignoring `#` lines inside fenced
/// code blocks.
pub fn headings(content: &str) -> Vec<Heading> {
    let mut in_fence = false;
    let mut found = Vec::new();
    for (index, line) in split_lines(content).into_iter().enumerate() {
        if is_fence(line) {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if let Some((level, text)) = parse_heading(line) {
            found.push(Heading {
                line: index + 1,
                level,
                text,
            });
        }
    }
    found
}

/// Find the section for `heading`, given as its text (`Goal`) or with its
/// level (`## Goal`). An exact text match wins; otherwise a case-insensitive
/// one is used. Zero or several matches are errors that name the options.
pub fn find_section(content: &str, heading: &str) -> Result<Section, ContentError> {
    let wanted = heading.trim();
    if wanted.is_empty() {
        return Err(ContentError::EmptyHeading);
    }
    let (wanted_level, wanted_text) = match parse_heading(wanted) {
        Some((level, text)) => (Some(level), text),
        None => (None, wanted.to_string()),
    };
    let all = headings(content);
    let level_ok = |h: &&Heading| wanted_level.is_none_or(|level| h.level == level);
    let mut matches: Vec<&Heading> = all
        .iter()
        .filter(level_ok)
        .filter(|h| h.text == wanted_text)
        .collect();
    if matches.is_empty() {
        let lower = wanted_text.to_lowercase();
        matches = all
            .iter()
            .filter(level_ok)
            .filter(|h| h.text.to_lowercase() == lower)
            .collect();
    }
    match matches.as_slice() {
        [] => Err(ContentError::HeadingNotFound {
            heading: wanted.to_string(),
            available: describe_headings(&all),
        }),
        [only] => {
            let found = (*only).clone();
            let total = split_lines(content).len();
            let end = all
                .iter()
                .find(|h| h.line > found.line && h.level <= found.level)
                .map(|h| h.line - 1)
                .unwrap_or(total);
            Ok(Section {
                range: LineRange {
                    start: found.line,
                    end,
                },
                heading: found,
            })
        }
        many => Err(ContentError::HeadingAmbiguous {
            heading: wanted_text,
            lines: many.iter().map(|h| h.line).collect(),
        }),
    }
}

/// A short list of headings for error messages, capped so a huge outline
/// does not flood the reply.
fn describe_headings(all: &[Heading]) -> String {
    const MAX: usize = 30;
    if all.is_empty() {
        return "(none)".to_string();
    }
    let mut listed: Vec<String> = all
        .iter()
        .take(MAX)
        .map(|h| format!("{} {}", "#".repeat(h.level as usize), h.text))
        .collect();
    if all.len() > MAX {
        listed.push(format!("… {} more", all.len() - MAX));
    }
    listed.join(", ")
}

/// Parse a 1-based line range: `12`, `12-40`, or `12-` (to the end), clamped
/// to the content. A start past the end is an error.
pub fn parse_line_range(range: &str, total_lines: usize) -> Result<LineRange, ContentError> {
    let invalid = |reason: &str| ContentError::InvalidLineRange {
        range: range.to_string(),
        reason: reason.to_string(),
    };
    let number = |raw: &str| {
        raw.trim()
            .parse::<usize>()
            .map_err(|_| invalid("expected N, N-M, or N-"))
    };
    let (start, end) = match range.split_once('-') {
        Some((start, "")) => (number(start)?, total_lines),
        Some((start, end)) => (number(start)?, number(end)?),
        None => {
            let line = number(range)?;
            (line, line)
        }
    };
    if start == 0 {
        return Err(invalid("lines start at 1"));
    }
    if end < start {
        return Err(invalid("end is before start"));
    }
    if start > total_lines {
        return Err(invalid(&format!("the item has {total_lines} lines")));
    }
    Ok(LineRange {
        start,
        end: end.min(total_lines),
    })
}

/// The lines in `range` prefixed with their line numbers and a tab, one per
/// line, the way `cat -n` shows a file.
pub fn numbered_lines(content: &str, range: LineRange) -> String {
    split_lines(content)
        .into_iter()
        .enumerate()
        .skip(range.start.saturating_sub(1))
        .take(range.end + 1 - range.start.max(1))
        .map(|(index, line)| format!("{}\t{}", index + 1, line))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Where a write changed the content: the first changed line (1-based, in
/// both the old and new body) and how many lines were removed and added there.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangedLines {
    pub start: usize,
    pub removed: usize,
    pub added: usize,
}

/// One targeted content change, applied on the server against the current body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum ContentEdit {
    /// Replace exact text. `old` must match once unless `replace_all` is set.
    Replace {
        old: String,
        new: String,
        #[serde(default)]
        replace_all: bool,
    },
    /// Append `text` as a new paragraph at the end.
    Append { text: String },
    /// Replace the body of one heading's section, keeping the heading line.
    ReplaceSection { heading: String, content: String },
}

/// Body of `POST /item/{id}/content`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentEditDto {
    #[serde(flatten)]
    pub edit: ContentEdit,
    /// Refuse the write unless the content still has this version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_version: Option<String>,
}

/// Compact reply to a content write.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentWriteDto {
    pub item_id: i64,
    pub title: String,
    pub version: String,
    pub content_chars: usize,
    /// Absent when the write left the content unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub changed_lines: Option<ChangedLines>,
}

/// Refuse the write when `expected` is given and the content has moved on.
pub fn check_version(content: &str, expected: Option<&str>) -> Result<(), ContentError> {
    match expected.map(str::trim).filter(|v| !v.is_empty()) {
        Some(expected) => {
            let actual = content_version(content);
            if actual == expected {
                Ok(())
            } else {
                Err(ContentError::VersionConflict {
                    expected: expected.to_string(),
                    actual,
                })
            }
        }
        None => Ok(()),
    }
}

/// Apply one edit to `content`, returning the new body.
pub fn apply_edit(content: &str, edit: &ContentEdit) -> Result<String, ContentError> {
    match edit {
        ContentEdit::Replace {
            old,
            new,
            replace_all,
        } => replace_text(content, old, new, *replace_all),
        ContentEdit::Append { text } => Ok(append_text(content, text)),
        ContentEdit::ReplaceSection {
            heading,
            content: body,
        } => replace_section(content, heading, body),
    }
}

/// Replace exact `old` text with `new`. Fails when `old` is empty or absent,
/// or matches more than once without `replace_all`. No regex, by design.
pub fn replace_text(
    content: &str,
    old: &str,
    new: &str,
    replace_all: bool,
) -> Result<String, ContentError> {
    if old.is_empty() {
        return Err(ContentError::EmptyOld);
    }
    let starts: Vec<usize> = content.match_indices(old).map(|(i, _)| i).collect();
    match starts.len() {
        0 => Err(ContentError::TextNotFound),
        1 => Ok(content.replacen(old, new, 1)),
        _ if replace_all => Ok(content.replace(old, new)),
        count => Err(ContentError::TextAmbiguous {
            count,
            lines: starts
                .iter()
                .map(|&i| content[..i].matches('\n').count() + 1)
                .collect(),
        }),
    }
}

/// Append `text` as a new paragraph: the body gets a trailing newline if it
/// lacks one, then the text and a newline.
pub fn append_text(content: &str, text: &str) -> String {
    let mut updated = content.to_string();
    if !updated.is_empty() && !updated.ends_with('\n') {
        updated.push('\n');
    }
    updated.push_str(text);
    updated.push('\n');
    updated
}

/// Replace the body of `heading`'s section (everything after the heading line
/// up to the next heading of the same or higher level, nested headings
/// included) with `body`. The heading line itself is kept.
pub fn replace_section(content: &str, heading: &str, body: &str) -> Result<String, ContentError> {
    let section = find_section(content, heading)?;
    let lines = split_lines(content);
    let mut out: Vec<&str> = Vec::with_capacity(lines.len());
    out.extend_from_slice(&lines[..section.range.start]);
    let body = body.strip_suffix('\n').unwrap_or(body);
    if !body.is_empty() {
        out.extend(body.split('\n'));
    }
    out.extend_from_slice(&lines[section.range.end..]);
    let mut updated = out.join("\n");
    if content.ends_with('\n') {
        updated.push('\n');
    }
    Ok(updated)
}

/// The block of lines that differs between `old` and `new`, found by
/// trimming the common leading and trailing lines. `None` when equal.
pub fn changed_lines(old: &str, new: &str) -> Option<ChangedLines> {
    if old == new {
        return None;
    }
    let a = split_lines(old);
    let b = split_lines(new);
    let prefix = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
    let max_suffix = a.len().min(b.len()) - prefix;
    let suffix = a
        .iter()
        .rev()
        .zip(b.iter().rev())
        .take(max_suffix)
        .take_while(|(x, y)| x == y)
        .count();
    Some(ChangedLines {
        start: prefix + 1,
        removed: a.len() - prefix - suffix,
        added: b.len() - prefix - suffix,
    })
}

/// Most context lines grep returns around each match.
pub const MAX_GREP_CONTEXT: usize = 10;
/// Most matching lines grep returns for one item; the rest are counted only.
pub const MAX_GREP_MATCHES_PER_ITEM: usize = 50;

/// Body of `POST /item/grep`: line-level search across item content.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrepQueryDto {
    /// Text to find, case-insensitive. A literal unless `regex` is true
    pub pattern: String,
    /// Treat `pattern` as a case-insensitive regular expression
    #[serde(default)]
    pub regex: bool,
    /// Only search descendants of this item (through `Parent` links)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub under: Option<i64>,
    /// Only search items with at least one of these types
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub types: Vec<String>,
    /// Lines of context before and after each match (0 to 10)
    #[serde(default)]
    pub context_lines: usize,
    /// Most items per page
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// Matching items to skip
    #[serde(default)]
    pub offset: i64,
}

/// One item's matching lines, formatted like `grep -n`: `12:matching line`,
/// `11-context line`, and `--` between separate groups.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrepHitDto {
    pub item_id: i64,
    pub title: String,
    /// Every matching line in the item, including any not listed
    pub match_count: usize,
    pub lines: Vec<String>,
    /// Set when more than `MAX_GREP_MATCHES_PER_ITEM` lines matched
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub truncated: bool,
}

/// A page of grep hits, one per matching item, in item id order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrepPageDto {
    pub count: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_offset: Option<i64>,
    pub items: Vec<GrepHitDto>,
}

/// The matching lines of `content` (grep-formatted, with up to
/// `context_lines` of context), how many lines matched in all, and whether
/// the listing stopped at `MAX_GREP_MATCHES_PER_ITEM`. `None` when nothing
/// matched.
pub fn grep_lines(
    content: &str,
    context_lines: usize,
    is_match: impl Fn(&str) -> bool,
) -> Option<(Vec<String>, usize, bool)> {
    let lines = split_lines(content);
    let matched: Vec<usize> = (0..lines.len()).filter(|&i| is_match(lines[i])).collect();
    if matched.is_empty() {
        return None;
    }
    let context = context_lines.min(MAX_GREP_CONTEXT);
    let truncated = matched.len() > MAX_GREP_MATCHES_PER_ITEM;
    let shown = &matched[..matched.len().min(MAX_GREP_MATCHES_PER_ITEM)];

    // Lines to print: each shown match and its context, merged where they
    // overlap, with `--` between separate groups when context is on.
    let mut printed: Vec<usize> = shown
        .iter()
        .flat_map(|&m| m.saturating_sub(context)..=(m + context).min(lines.len() - 1))
        .collect();
    printed.dedup();
    printed.sort_unstable();
    printed.dedup();

    let mut out = Vec::with_capacity(printed.len());
    for (n, &i) in printed.iter().enumerate() {
        if context > 0 && n > 0 && i > printed[n - 1] + 1 {
            out.push("--".to_string());
        }
        let marker = if shown.binary_search(&i).is_ok() {
            ':'
        } else {
            '-'
        };
        out.push(format!("{}{}{}", i + 1, marker, lines[i]));
    }
    Some((out, matched.len(), truncated))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replace_changes_only_the_matched_text() {
        let doc = "one\ntwo\nthree\n";
        let updated = replace_text(doc, "two", "2", false).unwrap();
        assert_eq!(updated, "one\n2\nthree\n");
        assert_eq!(
            changed_lines(doc, &updated),
            Some(ChangedLines {
                start: 2,
                removed: 1,
                added: 1
            })
        );
    }

    #[test]
    fn replace_errors_name_the_reason() {
        assert_eq!(
            replace_text("a", "", "b", false),
            Err(ContentError::EmptyOld)
        );
        assert_eq!(
            replace_text("a", "zz", "b", false),
            Err(ContentError::TextNotFound)
        );
        assert_eq!(
            replace_text("x\nTODO\ny\nTODO", "TODO", "done", false),
            Err(ContentError::TextAmbiguous {
                count: 2,
                lines: vec![2, 4]
            })
        );
        assert_eq!(
            replace_text("x\nTODO\ny\nTODO", "TODO", "done", true).unwrap(),
            "x\ndone\ny\ndone"
        );
    }

    #[test]
    fn replace_is_literal_not_regex() {
        assert_eq!(
            replace_text("a.b axb", "a.b", "ok", false).unwrap(),
            "ok axb"
        );
    }

    #[test]
    fn append_adds_a_paragraph() {
        assert_eq!(append_text("", "x"), "x\n");
        assert_eq!(append_text("a", "x"), "a\nx\n");
        assert_eq!(append_text("a\n", "x"), "a\nx\n");
    }

    #[test]
    fn replace_section_keeps_the_heading_and_the_rest() {
        let updated = replace_section(DOC, "Detail", "New detail.").unwrap();
        assert_eq!(
            updated,
            "Intro\n# Goal\nShip it.\n## Detail\nNew detail.\n## Other\nx\n# Notes\n```\n# not a heading\n```\nEnd.\n"
        );
        // Last section, no trailing newline in the original.
        assert_eq!(
            replace_section("# A\na\n# B\nb", "B", "c\n").unwrap(),
            "# A\na\n# B\nc"
        );
        // A middle section keeps the original's missing trailing newline.
        assert_eq!(
            replace_section("# A\na\n# B\nb", "A", "X").unwrap(),
            "# A\nX\n# B\nb"
        );
        // Empty body leaves the heading alone.
        assert_eq!(
            replace_section("# A\na\n# B\nb\n", "A", "").unwrap(),
            "# A\n# B\nb\n"
        );
        assert!(matches!(
            replace_section(DOC, "Nope", "x"),
            Err(ContentError::HeadingNotFound { .. })
        ));
    }

    #[test]
    fn version_check_refuses_a_stale_version() {
        assert!(check_version("a", None).is_ok());
        assert!(check_version("a", Some(&content_version("a"))).is_ok());
        assert!(matches!(
            check_version("b", Some(&content_version("a"))),
            Err(ContentError::VersionConflict { .. })
        ));
    }

    #[test]
    fn changed_lines_handles_inserts_and_deletes() {
        assert_eq!(changed_lines("a\nb", "a\nb"), None);
        assert_eq!(
            changed_lines("a\nc", "a\nb\nc"),
            Some(ChangedLines {
                start: 2,
                removed: 0,
                added: 1
            })
        );
        assert_eq!(
            changed_lines("a\nb\nc", "a\nc"),
            Some(ChangedLines {
                start: 2,
                removed: 1,
                added: 0
            })
        );
        assert_eq!(
            changed_lines("a\n", "a\nx\n"),
            Some(ChangedLines {
                start: 2,
                removed: 0,
                added: 1
            })
        );
    }

    #[test]
    fn edit_dto_round_trips_with_op_tag() {
        let dto: ContentEditDto = serde_json::from_value(serde_json::json!({
            "op": "replace", "old": "a", "new": "b", "expected_version": "v1"
        }))
        .unwrap();
        assert_eq!(
            dto.edit,
            ContentEdit::Replace {
                old: "a".into(),
                new: "b".into(),
                replace_all: false
            }
        );
        assert_eq!(dto.expected_version.as_deref(), Some("v1"));
    }

    #[test]
    fn grep_lines_formats_matches_with_context() {
        let doc = "a\nZ190 one\nb\nc\nd\ne\nZ190 two\nZ190 three\nf";
        let (lines, count, truncated) = grep_lines(doc, 1, |l| l.contains("Z190")).unwrap();
        assert_eq!(count, 3);
        assert!(!truncated);
        assert_eq!(
            lines,
            vec![
                "1-a",
                "2:Z190 one",
                "3-b",
                "--",
                "6-e",
                "7:Z190 two",
                "8:Z190 three",
                "9-f"
            ]
        );
        let (lines, _, _) = grep_lines(doc, 0, |l| l.contains("Z190")).unwrap();
        assert_eq!(lines, vec!["2:Z190 one", "7:Z190 two", "8:Z190 three"]);
        assert!(grep_lines(doc, 2, |l| l.contains("nope")).is_none());
    }

    #[test]
    fn grep_lines_merges_overlapping_context() {
        let doc = "x\nhit\ny\nhit\nz";
        let (lines, _, _) = grep_lines(doc, 2, |l| l == "hit").unwrap();
        assert_eq!(lines, vec!["1-x", "2:hit", "3-y", "4:hit", "5-z"]);
    }

    #[test]
    fn grep_lines_caps_matches_per_item() {
        let doc = "hit\n".repeat(MAX_GREP_MATCHES_PER_ITEM + 5);
        let (lines, count, truncated) = grep_lines(&doc, 0, |l| l == "hit").unwrap();
        assert_eq!(lines.len(), MAX_GREP_MATCHES_PER_ITEM);
        assert_eq!(count, MAX_GREP_MATCHES_PER_ITEM + 5);
        assert!(truncated);
    }

    const DOC: &str = "Intro\n# Goal\nShip it.\n## Detail\nMore.\n### Deep\nDeeper.\n## Other\nx\n# Notes\n```\n# not a heading\n```\nEnd.\n";

    #[test]
    fn split_lines_ignores_one_trailing_newline() {
        assert_eq!(split_lines(""), Vec::<&str>::new());
        assert_eq!(split_lines("\n"), vec![""]);
        assert_eq!(split_lines("a\nb\n"), vec!["a", "b"]);
        assert_eq!(split_lines("a\nb"), vec!["a", "b"]);
        assert_eq!(split_lines("a\n\n"), vec!["a", ""]);
    }

    #[test]
    fn headings_skip_fenced_code_and_non_headings() {
        let found: Vec<(usize, u8, String)> = headings(DOC)
            .into_iter()
            .map(|h| (h.line, h.level, h.text))
            .collect();
        assert_eq!(
            found,
            vec![
                (2, 1, "Goal".to_string()),
                (4, 2, "Detail".to_string()),
                (6, 3, "Deep".to_string()),
                (8, 2, "Other".to_string()),
                (10, 1, "Notes".to_string()),
            ]
        );
        assert!(headings("#hashtag\n####### seven").is_empty());
    }

    #[test]
    fn section_runs_to_next_heading_of_same_or_higher_level() {
        // A nested section stops at its sibling, keeping its own children.
        let detail = find_section(DOC, "Detail").unwrap();
        assert_eq!(detail.range, LineRange { start: 4, end: 7 });
        // A top-level section includes every nested heading.
        let goal = find_section(DOC, "# Goal").unwrap();
        assert_eq!(goal.range, LineRange { start: 2, end: 9 });
        // The last section runs to the end, past a fenced `#` line.
        let notes = find_section(DOC, "notes").unwrap();
        assert_eq!(notes.range, LineRange { start: 10, end: 14 });
    }

    #[test]
    fn section_errors_name_the_reason() {
        let missing = find_section(DOC, "Nope").unwrap_err();
        assert!(
            matches!(&missing, ContentError::HeadingNotFound { available, .. } if available.contains("## Detail"))
        );
        assert_eq!(
            find_section(DOC, "  ").unwrap_err(),
            ContentError::EmptyHeading
        );
        // A level that does not match is not found.
        assert!(matches!(
            find_section(DOC, "### Detail"),
            Err(ContentError::HeadingNotFound { .. })
        ));

        let twice = "# A\n## Tasks\nx\n# B\n## Tasks\ny";
        assert_eq!(
            find_section(twice, "Tasks").unwrap_err(),
            ContentError::HeadingAmbiguous {
                heading: "Tasks".to_string(),
                lines: vec![2, 5]
            }
        );
    }

    #[test]
    fn exact_case_wins_over_case_insensitive() {
        let doc = "# plan\na\n# Plan\nb";
        assert_eq!(find_section(doc, "Plan").unwrap().range.start, 3);
        assert!(matches!(
            find_section(doc, "PLAN"),
            Err(ContentError::HeadingAmbiguous { .. })
        ));
    }

    #[test]
    fn line_ranges_parse_and_clamp() {
        assert_eq!(
            parse_line_range("3", 10).unwrap(),
            LineRange { start: 3, end: 3 }
        );
        assert_eq!(
            parse_line_range("3-5", 10).unwrap(),
            LineRange { start: 3, end: 5 }
        );
        assert_eq!(
            parse_line_range("8-", 10).unwrap(),
            LineRange { start: 8, end: 10 }
        );
        assert_eq!(
            parse_line_range("8-99", 10).unwrap(),
            LineRange { start: 8, end: 10 }
        );
        for bad in ["0", "5-3", "11", "x", "1-y"] {
            assert!(parse_line_range(bad, 10).is_err(), "{bad}");
        }
    }

    #[test]
    fn numbered_lines_prefix_line_numbers() {
        assert_eq!(
            numbered_lines(DOC, LineRange { start: 4, end: 5 }),
            "4\t## Detail\n5\tMore."
        );
    }

    #[test]
    fn version_is_stable_and_short() {
        assert_eq!(content_version("hello"), content_version("hello"));
        assert_eq!(content_version("hello").len(), VERSION_HEX_CHARS);
        // Known SHA-256 prefix, so the version never drifts between releases.
        assert_eq!(content_version(""), "e3b0c44298fc1c14");
    }

    #[test]
    fn version_changes_with_any_edit() {
        assert_ne!(content_version("a\nb"), content_version("a\nb\n"));
        assert_ne!(content_version("Title"), content_version("title"));
    }
}

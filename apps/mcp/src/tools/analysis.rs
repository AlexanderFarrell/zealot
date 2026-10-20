use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use chrono::NaiveDate;
use rmcp::{ErrorData as McpError, handler::server::wrapper::Parameters, model::CallToolResult};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;
use zealot_domain::{item::ItemDto, repeat::RepeatEntryDto};

use crate::{
    output::{self, Detail},
    tools::{NoParams, ZealotServer, err_ctx, local_today, parse_date},
};

const FETCH_PAGE_SIZE: i64 = 100;
const FETCH_CAP: usize = 1000;

#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WikiStatsParams {
    /// Include the per-type counts (default false; `list_item_types` returns the same data)
    pub include_types: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LimitParams {
    /// Max items to return (default 50, max 1000)
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HabitStatsParams {
    /// Start date (inclusive), YYYY-MM-DD
    pub start_date: String,
    /// End date (inclusive), YYYY-MM-DD
    pub end_date: String,
}

async fn fetch_all_items(server: &ZealotServer) -> Result<(Vec<ItemDto>, bool), McpError> {
    let mut items = Vec::new();
    let mut offset = 0_i64;

    while items.len() < FETCH_CAP {
        let remaining = FETCH_CAP - items.len();
        let limit = FETCH_PAGE_SIZE.min(remaining as i64);
        let page = server
            .client
            .recent_items(limit, offset)
            .await
            .map_err(|e| err_ctx("items", e))?;
        let page_len = page.len();
        items.extend(page);

        if page_len < limit as usize {
            return Ok((items, false));
        }
        offset += limit;
    }

    Ok((items, true))
}

fn completion_rate(complete: usize, not_complete: usize) -> Option<f64> {
    let denom = complete + not_complete;
    if denom == 0 {
        None
    } else {
        let rate = complete as f64 / denom as f64;
        Some((rate * 10_000.0).round() / 10_000.0)
    }
}

fn status_key(status: &str) -> &'static str {
    match status.trim() {
        "Complete" => "complete",
        "Skip" => "skip",
        "Alternate" => "alternate",
        "Not Complete" | "NotComplete" => "not_complete",
        _ => "other",
    }
}

fn parse_entry_date(entry: &RepeatEntryDto) -> Result<NaiveDate, McpError> {
    parse_date(&entry.date).map_err(|_| {
        McpError::internal_error(
            format!("habit entry has invalid date from upstream: {}", entry.date),
            None,
        )
    })
}

#[rmcp::tool_router(router = analysis_tool_router, vis = "pub")]
impl ZealotServer {
    #[rmcp::tool(
        description = "High-level wiki statistics: totals for item types and attribute kinds, and recently modified items. Per-type counts are omitted unless `include_types: true` (they duplicate `list_item_types`). Returns summaries, not full item content."
    )]
    pub async fn wiki_stats(
        &self,
        Parameters(p): Parameters<WikiStatsParams>,
    ) -> Result<CallToolResult, McpError> {
        let (types, attribute_kinds, recent) = tokio::join!(
            self.client.item_type_summaries(),
            self.client.list_attribute_kinds(),
            self.client.recent_items(10, 0),
        );
        let types = types.map_err(|e| err_ctx("item types", e))?;
        let attribute_kinds = attribute_kinds.map_err(|e| err_ctx("attribute kinds", e))?;
        let recent = recent.map_err(|e| err_ctx("recent items", e))?;

        let type_assignment_count: i64 = types.iter().map(|t| t.item_count).sum();
        let recently_modified: Vec<_> = recent
            .iter()
            .map(|item| output::ItemSummary::project(item, Detail::Meta))
            .collect();

        let mut stats = json!({
            "totals": {
                "item_types": types.len(),
                "type_assignment_count": type_assignment_count,
                "attribute_kinds": attribute_kinds.len(),
            },
            "recently_modified": recently_modified,
        });
        if p.include_types.unwrap_or(false) {
            stats["per_type"] = json!(types);
        }
        Ok(output::json_result(&stats))
    }

    #[rmcp::tool(
        description = "Find orphaned wiki items from a sampled item graph. An orphan has no outgoing links and no other sampled item links to it. Scans up to 1000 recent items and reports whether results may be truncated."
    )]
    pub async fn orphaned_items(
        &self,
        Parameters(p): Parameters<LimitParams>,
    ) -> Result<CallToolResult, McpError> {
        let limit = p.limit.unwrap_or(50).clamp(1, FETCH_CAP);
        let (items, truncated) = fetch_all_items(self).await?;

        let incoming: HashSet<i64> = items
            .iter()
            .flat_map(|item| item.links.iter().map(|link| link.other_item_id))
            .collect();

        let mut orphaned: Vec<_> = items
            .iter()
            .filter(|item| item.links.is_empty() && !incoming.contains(&item.item_id))
            .map(|item| output::ItemSummary::project(item, Detail::Summary))
            .collect();
        orphaned.sort_by(|a, b| a.title.cmp(&b.title).then(a.item_id.cmp(&b.item_id)));
        let total = orphaned.len();
        orphaned.truncate(limit);

        Ok(output::json_result(&json!({
            "count": orphaned.len(),
            "matching_count": total,
            "truncated": truncated,
            "items": orphaned,
        })))
    }

    #[rmcp::tool(
        description = "Analyze attribute usage across up to 1000 recent items. Reports counts per used key, unused defined attribute kinds, and undefined keys found on items."
    )]
    pub async fn attribute_usage(
        &self,
        Parameters(_): Parameters<NoParams>,
    ) -> Result<CallToolResult, McpError> {
        let (items, kinds) =
            tokio::join!(fetch_all_items(self), self.client.list_attribute_kinds());
        let (items, truncated) = items?;
        let kinds = kinds.map_err(|e| err_ctx("attribute kinds", e))?;

        let mut usage: BTreeMap<String, usize> = BTreeMap::new();
        for item in &items {
            if let Some(attrs) = item.attributes.as_object() {
                for key in attrs.keys() {
                    *usage.entry(key.clone()).or_default() += 1;
                }
            }
        }

        let defined: BTreeSet<String> = kinds.iter().map(|kind| kind.key.clone()).collect();
        let used: BTreeSet<String> = usage.keys().cloned().collect();
        let unused_defined: Vec<_> = defined.difference(&used).cloned().collect();
        let undefined_used: Vec<_> = used.difference(&defined).cloned().collect();
        let usage_rows: Vec<_> = usage
            .into_iter()
            .map(|(key, count)| json!({"key": key, "item_count": count}))
            .collect();

        Ok(output::json_result(&json!({
            "scanned_items": items.len(),
            "truncated": truncated,
            "usage": usage_rows,
            "unused_defined_kinds": unused_defined,
            "undefined_used_keys": undefined_used,
        })))
    }

    #[rmcp::tool(
        description = "Summarize habit completion across a date range. Range is capped at 366 inclusive days. Complete extends streaks; Skip and Alternate are neutral; Not Complete breaks streaks. Days after today are not counted, and an unlogged today is pending rather than Not Complete; `counted_through` is the last day counted."
    )]
    pub async fn habit_stats(
        &self,
        Parameters(p): Parameters<HabitStatsParams>,
    ) -> Result<CallToolResult, McpError> {
        self.habit_stats_as_of(p, local_today()).await
    }
}

impl ZealotServer {
    /// `habit_stats` evaluated as if the user's local date were `today`.
    ///
    /// Only days up to `today` are fetched and counted. Not Complete has no
    /// stored row, so an unlogged today reads as Not Complete; it is treated
    /// as pending until the day is over.
    pub async fn habit_stats_as_of(
        &self,
        p: HabitStatsParams,
        today: NaiveDate,
    ) -> Result<CallToolResult, McpError> {
        let start = parse_date(&p.start_date)?;
        let end = parse_date(&p.end_date)?;
        if end < start {
            return Err(McpError::invalid_params(
                "end_date must be on or after start_date",
                None,
            ));
        }
        if (end - start).num_days() > 365 {
            return Err(McpError::invalid_params(
                "habit_stats range is capped at 366 inclusive days",
                None,
            ));
        }

        let counted_end = end.min(today);
        let entries = if counted_end < start {
            Vec::new()
        } else {
            self.client
                .repeats_for_range(start, counted_end)
                .await
                .map_err(|e| err_ctx("habit entries", e))?
        };
        let rows = habit_rows(entries, today)?;

        Ok(output::json_result(&json!({
            "start_date": p.start_date,
            "end_date": p.end_date,
            "counted_through": (counted_end >= start).then(|| counted_end.to_string()),
            "habit_count": rows.len(),
            "habits": rows,
        })))
    }
}

/// Whether an entry counts toward stats on `today`: past days always count,
/// today counts once it has a logged status, and future days never count.
fn counts_toward_stats(date: NaiveDate, status: &str, today: NaiveDate) -> bool {
    date < today || (date == today && status_key(status) != "not_complete")
}

/// Per-habit counts, completion rate, and streaks for the counted entries.
fn habit_rows(
    entries: Vec<RepeatEntryDto>,
    today: NaiveDate,
) -> Result<Vec<serde_json::Value>, McpError> {
    let mut by_item: BTreeMap<i64, Vec<(NaiveDate, RepeatEntryDto)>> = BTreeMap::new();
    for entry in entries {
        let date = parse_entry_date(&entry)?;
        if !counts_toward_stats(date, &entry.status, today) {
            continue;
        }
        by_item
            .entry(entry.item.item_id)
            .or_default()
            .push((date, entry));
    }

    let mut rows = Vec::new();
    for (item_id, mut entries) in by_item {
        entries.sort_by_key(|(date, _)| *date);
        let title = entries
            .first()
            .map(|(_, entry)| entry.item.title.clone())
            .unwrap_or_default();
        let mut counts: HashMap<&'static str, usize> = HashMap::from([
            ("complete", 0),
            ("skip", 0),
            ("alternate", 0),
            ("not_complete", 0),
            ("other", 0),
        ]);
        let mut current_streak = 0_usize;
        let mut longest_streak = 0_usize;

        for (_, entry) in &entries {
            let key = status_key(&entry.status);
            *counts.entry(key).or_default() += 1;
            match key {
                "complete" => {
                    current_streak += 1;
                    longest_streak = longest_streak.max(current_streak);
                }
                "skip" | "alternate" => {}
                _ => current_streak = 0,
            }
        }

        rows.push(json!({
            "item_id": item_id,
            "title": title,
            "entries": entries.len(),
            "counts": {
                "complete": counts["complete"],
                "skip": counts["skip"],
                "alternate": counts["alternate"],
                "not_complete": counts["not_complete"],
                "other": counts["other"],
            },
            "completion_rate": completion_rate(counts["complete"], counts["not_complete"]),
            "current_streak": current_streak,
            "longest_streak": longest_streak,
        }));
    }
    Ok(rows)
}

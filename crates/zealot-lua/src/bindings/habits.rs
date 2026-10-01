//! Read access to Repeat (habit) entries and per-habit summaries, matching
//! the MCP `list_habits`, `get_habit_entries` and `habit_stats` tools.

use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

use chrono::NaiveDate;
use chrono_tz::Tz;
use mlua::{Lua, Table};
use zealot_app::services::ZealotServices;
use zealot_domain::{common::id::Id, repeat::RepeatEntry, repeat::RepeatStatus};

use super::{
    clock::{local_today, parse_date},
    items::{item_to_lua, make_account},
};

const MAX_STATS_DAYS: i64 = 366;

fn runtime(e: impl ToString) -> mlua::Error {
    mlua::Error::RuntimeError(e.to_string())
}

pub fn register(
    zealot: &Table,
    lua: &Lua,
    services: Arc<ZealotServices>,
    account_id: Id,
    tz: Tz,
) -> mlua::Result<()> {
    let habits = lua.create_table()?;

    // zealot.habits.list() -> item array of every Repeat item
    {
        let svc = services.clone();
        habits.set(
            "list",
            lua.create_async_function(move |lua, ()| {
                let svc = svc.clone();
                async move {
                    let account = make_account(account_id)?;
                    let items = svc.repeat.get_all_repeat_items(&account).map_err(runtime)?;
                    let result = lua.create_table()?;
                    for (i, item) in items.iter().enumerate() {
                        result.set(i + 1, item_to_lua(&lua, item)?)?;
                    }
                    Ok(result)
                }
            })?,
        )?;
    }

    // zealot.habits.entries(start_date, end_date?) -> {item_id,title,date,status,comment}[]
    {
        let svc = services.clone();
        habits.set(
            "entries",
            lua.create_async_function(move |lua, (start, end): (String, Option<String>)| {
                let svc = svc.clone();
                async move {
                    let account = make_account(account_id)?;
                    let start = parse_date(&start)?;
                    let end = end.as_deref().map(parse_date).transpose()?.unwrap_or(start);
                    if end < start {
                        return Err(runtime("end_date must be on or after start_date"));
                    }
                    let entries = svc
                        .repeat
                        .get_for_range(&start, &end, &account)
                        .await
                        .map_err(runtime)?;
                    let result = lua.create_table()?;
                    for (i, entry) in entries.iter().enumerate() {
                        let row = lua.create_table()?;
                        row.set("item_id", i64::from(entry.item.item_id))?;
                        row.set("title", entry.item.title.clone())?;
                        row.set("date", entry.date.format("%Y-%m-%d").to_string())?;
                        row.set("status", entry.status.to_string())?;
                        row.set("comment", entry.comment.clone())?;
                        result.set(i + 1, row)?;
                    }
                    Ok(result)
                }
            })?,
        )?;
    }

    // zealot.habits.stats(start_date, end_date?) -> summary table
    {
        let svc = services.clone();
        habits.set(
            "stats",
            lua.create_async_function(move |lua, (start, end): (String, Option<String>)| {
                let svc = svc.clone();
                async move {
                    let account = make_account(account_id)?;
                    let start = parse_date(&start)?;
                    let end = end.as_deref().map(parse_date).transpose()?.unwrap_or(start);
                    if end < start {
                        return Err(runtime("end_date must be on or after start_date"));
                    }
                    if (end - start).num_days() >= MAX_STATS_DAYS {
                        return Err(runtime(
                            "habits.stats range is capped at 366 inclusive days",
                        ));
                    }
                    let today = local_today(tz);
                    let counted_end = end.min(today);
                    let entries = if counted_end < start {
                        Vec::new()
                    } else {
                        svc.repeat
                            .get_for_range(&start, &counted_end, &account)
                            .await
                            .map_err(runtime)?
                    };
                    let rows = habit_rows(&entries, today);

                    let result = lua.create_table()?;
                    result.set("start_date", start.format("%Y-%m-%d").to_string())?;
                    result.set("end_date", end.format("%Y-%m-%d").to_string())?;
                    if counted_end >= start {
                        result.set(
                            "counted_through",
                            counted_end.format("%Y-%m-%d").to_string(),
                        )?;
                    }
                    result.set("habit_count", rows.len())?;
                    let habits = lua.create_table()?;
                    for (i, row) in rows.iter().enumerate() {
                        let t = lua.create_table()?;
                        t.set("item_id", row.item_id)?;
                        t.set("title", row.title.clone())?;
                        t.set("entries", row.entries)?;
                        let counts = lua.create_table()?;
                        for (key, value) in &row.counts {
                            counts.set(*key, *value)?;
                        }
                        t.set("counts", counts)?;
                        if let Some(rate) = row.completion_rate {
                            t.set("completion_rate", rate)?;
                        }
                        t.set("current_streak", row.current_streak)?;
                        t.set("longest_streak", row.longest_streak)?;
                        habits.set(i + 1, t)?;
                    }
                    result.set("habits", habits)?;
                    Ok(result)
                }
            })?,
        )?;
    }

    zealot.set("habits", habits)?;
    Ok(())
}

#[derive(Debug, PartialEq)]
pub struct HabitRow {
    pub item_id: i64,
    pub title: String,
    pub entries: usize,
    pub counts: Vec<(&'static str, usize)>,
    pub completion_rate: Option<f64>,
    pub current_streak: usize,
    pub longest_streak: usize,
}

fn status_key(status: RepeatStatus) -> &'static str {
    match status {
        RepeatStatus::Complete => "complete",
        RepeatStatus::Skip => "skip",
        RepeatStatus::Alternate => "alternate",
        RepeatStatus::NotComplete => "not_complete",
    }
}

/// Past days always count, today counts once it has a logged status, future
/// days never count (same rule as MCP `habit_stats`).
fn counts_toward_stats(date: NaiveDate, status: RepeatStatus, today: NaiveDate) -> bool {
    date < today || (date == today && status != RepeatStatus::NotComplete)
}

pub fn habit_rows(entries: &[RepeatEntry], today: NaiveDate) -> Vec<HabitRow> {
    let mut by_item: BTreeMap<i64, Vec<&RepeatEntry>> = BTreeMap::new();
    for entry in entries {
        if counts_toward_stats(entry.date, entry.status, today) {
            by_item
                .entry(i64::from(entry.item.item_id))
                .or_default()
                .push(entry);
        }
    }

    by_item
        .into_iter()
        .map(|(item_id, mut entries)| {
            entries.sort_by_key(|e| e.date);
            let mut counts: HashMap<&'static str, usize> = HashMap::from([
                ("complete", 0),
                ("skip", 0),
                ("alternate", 0),
                ("not_complete", 0),
            ]);
            let (mut current, mut longest) = (0_usize, 0_usize);
            for entry in &entries {
                *counts.entry(status_key(entry.status)).or_default() += 1;
                match entry.status {
                    RepeatStatus::Complete => {
                        current += 1;
                        longest = longest.max(current);
                    }
                    RepeatStatus::Skip | RepeatStatus::Alternate => {}
                    RepeatStatus::NotComplete => current = 0,
                }
            }
            let denom = counts["complete"] + counts["not_complete"];
            HabitRow {
                item_id,
                title: entries[0].item.title.clone(),
                entries: entries.len(),
                completion_rate: (denom > 0).then(|| {
                    (counts["complete"] as f64 / denom as f64 * 10_000.0).round() / 10_000.0
                }),
                counts: ["complete", "skip", "alternate", "not_complete"]
                    .into_iter()
                    .map(|k| (k, counts[k]))
                    .collect(),
                current_streak: current,
                longest_streak: longest,
            }
        })
        .collect()
}

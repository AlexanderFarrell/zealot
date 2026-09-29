use std::sync::Arc;

use chrono::{DateTime, Utc};
use mlua::{Lua, Table};
use zealot_app::services::ZealotServices;
use zealot_domain::{
    common::id::Id,
    statistic::{
        CreateStatisticEntryDto, StatisticEntryDto, StatisticSummaryDto, UpdateStatisticEntryDto,
    },
};

use super::items::{item_to_lua, make_account};

pub fn register(
    zealot: &Table,
    lua: &Lua,
    services: Arc<ZealotServices>,
    account_id: Id,
) -> mlua::Result<()> {
    let statistics = lua.create_table()?;

    {
        let svc = services.clone();
        statistics.set(
            "list",
            lua.create_async_function(move |lua, parent_id: Option<i64>| {
                let svc = svc.clone();
                async move {
                    let account = make_account(account_id)?;
                    let parent_id = parent_id.map(Id::try_from).transpose().map_err(runtime)?;
                    let items = svc
                        .statistic
                        .list_items(parent_id, &account)
                        .map_err(runtime)?;
                    let result = lua.create_table()?;
                    for (index, item) in items.iter().enumerate() {
                        result.set(index + 1, item_to_lua(&lua, item)?)?;
                    }
                    Ok(result)
                }
            })?,
        )?;
    }

    {
        let svc = services.clone();
        statistics.set(
            "entries",
            lua.create_async_function(move |lua, (item_id, opts): (i64, Option<Table>)| {
                let svc = svc.clone();
                async move {
                    let account = make_account(account_id)?;
                    let item_id = Id::try_from(item_id).map_err(runtime)?;
                    let (start, end) = range(&opts)?;
                    let limit = opts
                        .as_ref()
                        .and_then(|table| table.get::<i64>("limit").ok())
                        .unwrap_or(50)
                        .clamp(1, 100);
                    let offset = opts
                        .as_ref()
                        .and_then(|table| table.get::<i64>("offset").ok())
                        .unwrap_or(0)
                        .max(0);
                    let page = svc
                        .statistic
                        .list_entries(item_id, start, end, limit, offset, &account)
                        .map_err(runtime)?;
                    let result = lua.create_table()?;
                    result.set("count", page.count)?;
                    result.set("next_offset", page.next_offset)?;
                    let entries = lua.create_table()?;
                    for (index, entry) in page.entries.iter().enumerate() {
                        entries.set(index + 1, entry_to_lua(&lua, entry)?)?;
                    }
                    result.set("entries", entries)?;
                    Ok(result)
                }
            })?,
        )?;
    }

    {
        let svc = services.clone();
        statistics.set(
            "daily",
            lua.create_async_function(move |lua, (item_id, opts): (i64, Option<Table>)| {
                let svc = svc.clone();
                async move {
                    let account = make_account(account_id)?;
                    let item_id = Id::try_from(item_id).map_err(runtime)?;
                    let (start, end) = range(&opts)?;
                    let points = svc
                        .statistic
                        .daily(item_id, start, end, &account)
                        .map_err(runtime)?;
                    let result = lua.create_table()?;
                    for (index, point) in points.iter().enumerate() {
                        let row = lua.create_table()?;
                        row.set("date", point.date.clone())?;
                        row.set("value", point.value)?;
                        row.set("count", point.count)?;
                        result.set(index + 1, row)?;
                    }
                    Ok(result)
                }
            })?,
        )?;
    }

    {
        let svc = services.clone();
        statistics.set(
            "summary",
            lua.create_async_function(move |lua, (item_id, opts): (i64, Option<Table>)| {
                let svc = svc.clone();
                async move {
                    let account = make_account(account_id)?;
                    let item_id = Id::try_from(item_id).map_err(runtime)?;
                    let (start, end) = range(&opts)?;
                    let summary = svc
                        .statistic
                        .summary(item_id, start, end, &account)
                        .map_err(runtime)?;
                    summary_to_lua(&lua, &summary)
                }
            })?,
        )?;
    }

    {
        let svc = services.clone();
        statistics.set(
            "record",
            lua.create_async_function(
                move |lua, (item_id, value, opts): (i64, f64, Option<Table>)| {
                    let svc = svc.clone();
                    async move {
                        let account = make_account(account_id)?;
                        let item_id = Id::try_from(item_id).map_err(runtime)?;
                        let dto = CreateStatisticEntryDto {
                            value,
                            occurred_at: option_string(&opts, "occurred_at")?,
                            related_item_id: opts
                                .as_ref()
                                .and_then(|table| table.get::<i64>("related_item_id").ok()),
                            comment: option_string(&opts, "comment")?,
                        };
                        let entry = svc
                            .statistic
                            .create(item_id, &dto, &account)
                            .map_err(runtime)?;
                        entry_to_lua(&lua, &StatisticEntryDto::from(&entry))
                    }
                },
            )?,
        )?;
    }

    {
        let svc = services.clone();
        statistics.set(
            "update",
            lua.create_async_function(move |lua, (entry_id, patch): (i64, Table)| {
                let svc = svc.clone();
                async move {
                    let account = make_account(account_id)?;
                    let entry_id = Id::try_from(entry_id).map_err(runtime)?;
                    let related_item_id =
                        if patch.get::<bool>("clear_related_item").unwrap_or(false) {
                            Some(None)
                        } else {
                            patch.get::<i64>("related_item_id").ok().map(Some)
                        };
                    let comment = if patch.get::<bool>("clear_comment").unwrap_or(false) {
                        Some(None)
                    } else {
                        patch.get::<String>("comment").ok().map(Some)
                    };
                    let dto = UpdateStatisticEntryDto {
                        value: patch.get::<f64>("value").ok(),
                        occurred_at: patch.get::<String>("occurred_at").ok(),
                        related_item_id,
                        comment,
                    };
                    let entry = svc
                        .statistic
                        .update(entry_id, &dto, &account)
                        .map_err(runtime)?;
                    entry_to_lua(&lua, &StatisticEntryDto::from(&entry))
                }
            })?,
        )?;
    }

    {
        let svc = services;
        statistics.set(
            "delete",
            lua.create_async_function(move |_, entry_id: i64| {
                let svc = svc.clone();
                async move {
                    let account = make_account(account_id)?;
                    let entry_id = Id::try_from(entry_id).map_err(runtime)?;
                    svc.statistic.delete(entry_id, &account).map_err(runtime)?;
                    Ok(true)
                }
            })?,
        )?;
    }

    zealot.set("statistics", statistics)?;
    Ok(())
}

fn runtime(error: impl ToString) -> mlua::Error {
    mlua::Error::RuntimeError(error.to_string())
}

fn option_string(opts: &Option<Table>, key: &str) -> mlua::Result<Option<String>> {
    Ok(opts
        .as_ref()
        .and_then(|table| table.get::<String>(key).ok()))
}

fn range(opts: &Option<Table>) -> mlua::Result<(Option<DateTime<Utc>>, Option<DateTime<Utc>>)> {
    Ok((
        parse_time(option_string(opts, "start")?)?,
        parse_time(option_string(opts, "end")?)?,
    ))
}

fn parse_time(value: Option<String>) -> mlua::Result<Option<DateTime<Utc>>> {
    value
        .map(|value| {
            DateTime::parse_from_rfc3339(&value)
                .map(|value| value.with_timezone(&Utc))
                .map_err(runtime)
        })
        .transpose()
}

fn entry_to_lua(lua: &Lua, entry: &StatisticEntryDto) -> mlua::Result<Table> {
    let row = lua.create_table()?;
    row.set("statistic_entry_id", entry.statistic_entry_id)?;
    row.set("item_id", entry.item_id)?;
    row.set("value", entry.value)?;
    row.set("occurred_at", entry.occurred_at.clone())?;
    row.set("related_item_id", entry.related_item_id)?;
    row.set("comment", entry.comment.clone())?;
    row.set("created_at", entry.created_at.clone())?;
    row.set("updated_at", entry.updated_at.clone())?;
    Ok(row)
}

fn summary_to_lua(lua: &Lua, summary: &StatisticSummaryDto) -> mlua::Result<Table> {
    let row = lua.create_table()?;
    row.set("count", summary.count)?;
    row.set("minimum", summary.minimum)?;
    row.set("maximum", summary.maximum)?;
    row.set("average", summary.average)?;
    row.set("sum", summary.sum)?;
    row.set("delta", summary.delta)?;
    if let Some(point) = &summary.first {
        let value = lua.create_table()?;
        value.set("value", point.value)?;
        value.set("occurred_at", point.occurred_at.clone())?;
        row.set("first", value)?;
    }
    if let Some(point) = &summary.latest {
        let value = lua.create_table()?;
        value.set("value", point.value)?;
        value.set("occurred_at", point.occurred_at.clone())?;
        row.set("latest", value)?;
    }
    Ok(row)
}

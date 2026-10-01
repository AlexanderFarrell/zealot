//! Read access to planner time blocks, matching MCP `get_time_blocks`.

use std::sync::Arc;

use mlua::{Lua, Table};
use zealot_app::services::ZealotServices;
use zealot_domain::{common::id::Id, time_block::TimeBlock};

use super::{clock::parse_date, items::make_account};

fn runtime(e: impl ToString) -> mlua::Error {
    mlua::Error::RuntimeError(e.to_string())
}

/// `H:MM`, as MCP renders time block times.
fn format_clock(min: i32) -> String {
    format!("{}:{:02}", min / 60, min % 60)
}

fn blocks_to_lua(lua: &Lua, blocks: &[TimeBlock]) -> mlua::Result<Table> {
    let result = lua.create_table()?;
    for (i, block) in blocks.iter().enumerate() {
        let row = lua.create_table()?;
        row.set("block_id", i64::from(block.block_id))?;
        row.set("item_id", i64::from(block.item.item_id))?;
        row.set("title", block.item.title.clone())?;
        row.set("date", block.date.format("%Y-%m-%d").to_string())?;
        row.set("start", format_clock(block.start_min))?;
        row.set("end", format_clock(block.end_min))?;
        row.set("start_min", block.start_min)?;
        row.set("end_min", block.end_min)?;
        row.set("minutes", block.end_min - block.start_min)?;
        row.set("note", block.note.clone())?;
        result.set(i + 1, row)?;
    }
    Ok(result)
}

pub fn register(
    zealot: &Table,
    lua: &Lua,
    services: Arc<ZealotServices>,
    account_id: Id,
) -> mlua::Result<()> {
    let time_blocks = lua.create_table()?;

    // zealot.time_blocks.for_range(start_date, end_date?) -> block array
    {
        let svc = services.clone();
        time_blocks.set(
            "for_range",
            lua.create_async_function(move |lua, (start, end): (String, Option<String>)| {
                let svc = svc.clone();
                async move {
                    let account = make_account(account_id)?;
                    let start = parse_date(&start)?;
                    let end = end.as_deref().map(parse_date).transpose()?.unwrap_or(start);
                    if end < start {
                        return Err(runtime("end_date must be on or after start_date"));
                    }
                    let blocks = svc
                        .time_block
                        .get_for_range(&start, &end, &account)
                        .await
                        .map_err(runtime)?;
                    blocks_to_lua(&lua, &blocks)
                }
            })?,
        )?;
    }

    // zealot.time_blocks.for_item(item_id) -> block array across all dates
    {
        let svc = services.clone();
        time_blocks.set(
            "for_item",
            lua.create_async_function(move |lua, item_id: i64| {
                let svc = svc.clone();
                async move {
                    let account = make_account(account_id)?;
                    let item_id = Id::try_from(item_id).map_err(runtime)?;
                    let blocks = svc
                        .time_block
                        .get_for_item(item_id, &account)
                        .await
                        .map_err(runtime)?;
                    blocks_to_lua(&lua, &blocks)
                }
            })?,
        )?;
    }

    zealot.set("time_blocks", time_blocks)?;
    Ok(())
}

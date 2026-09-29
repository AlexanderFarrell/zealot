use std::sync::{Arc, Mutex};

use mlua::{Lua, Table};
use zealot_app::ports::{events::ZealotEvent, rule_runner::RuleContext};
use zealot_domain::common::id::Id;

pub fn register(
    zealot: &Table,
    lua: &Lua,
    context: &RuleContext,
    _account_id: Id,
    output_buf: Arc<Mutex<Vec<String>>>,
) -> mlua::Result<()> {
    let buf_clone = output_buf.clone();
    let notify = lua.create_function(move |_, msg: String| {
        buf_clone.lock().unwrap().push(msg);
        Ok(())
    })?;
    zealot.set("notify", notify.clone())?;
    zealot.set("log", notify)?;

    zealot.set("now", chrono::Utc::now().to_rfc3339())?;
    zealot.set("date", chrono::Local::now().format("%Y-%m-%d").to_string())?;

    match context {
        RuleContext::Event(event) => {
            let event_table = event_to_lua(lua, event)?;
            zealot.set("event", event_table)?;
        }
        _ => {
            zealot.set("event", mlua::Value::Nil)?;
        }
    }

    Ok(())
}

fn event_to_lua(lua: &Lua, event: &ZealotEvent) -> mlua::Result<Table> {
    let t = lua.create_table()?;
    t.set("kind", event.trigger_kind())?;
    match event {
        ZealotEvent::ItemCreated { item, .. } => {
            t.set("item_id", i64::from(item.item_id))?;
            t.set("title", item.title.clone())?;
        }
        ZealotEvent::ItemUpdated { item, .. } => {
            t.set("item_id", i64::from(item.item_id))?;
            t.set("title", item.title.clone())?;
        }
        ZealotEvent::ItemDeleted { item_id, .. } => {
            t.set("item_id", i64::from(*item_id))?;
        }
        ZealotEvent::CommentAdded {
            item_id, comment, ..
        } => {
            t.set("item_id", i64::from(*item_id))?;
            t.set("comment_id", i64::from(comment.comment_id))?;
            t.set("content", comment.content.clone())?;
        }
        ZealotEvent::TypeAssigned {
            item, type_name, ..
        } => {
            t.set("item_id", i64::from(item.item_id))?;
            t.set("type_name", type_name.clone())?;
        }
        ZealotEvent::TypeUnassigned {
            item, type_name, ..
        } => {
            t.set("item_id", i64::from(item.item_id))?;
            t.set("type_name", type_name.clone())?;
        }
        ZealotEvent::AttributeSet {
            item,
            attribute_key,
            ..
        } => {
            t.set("item_id", i64::from(item.item_id))?;
            t.set("attribute_key", attribute_key.clone())?;
        }
    }
    Ok(t)
}

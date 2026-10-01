use std::sync::{Arc, Mutex};

use super::items::{item_to_lua, json_to_lua};
use mlua::{Lua, Table, Value as LuaValue};
use zealot_app::ports::{events::ZealotEvent, rule_runner::RuleContext};
use zealot_domain::{attribute::Attribute, common::id::Id, item::Item};

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

/// Builds `zealot.event`. Every kind carries `kind`; kinds that carry an item
/// also provide `item` (the full item table) and `item_id`.
fn event_to_lua(lua: &Lua, event: &ZealotEvent) -> mlua::Result<Table> {
    let t = lua.create_table()?;
    t.set("kind", event.trigger_kind())?;
    match event {
        ZealotEvent::ItemCreated { item, .. } | ZealotEvent::ItemUpdated { item, .. } => {
            set_item(lua, &t, item)?;
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
            t.set(
                "timestamp",
                comment.timestamp.format("%Y-%m-%dT%H:%M:%S").to_string(),
            )?;
        }
        ZealotEvent::TypeAssigned {
            item, type_name, ..
        }
        | ZealotEvent::TypeUnassigned {
            item, type_name, ..
        } => {
            set_item(lua, &t, item)?;
            t.set("type_name", type_name.clone())?;
        }
        ZealotEvent::AttributeSet {
            item,
            attribute_key,
            old_value,
            new_value,
            ..
        } => {
            set_item(lua, &t, item)?;
            t.set("attribute_key", attribute_key.clone())?;
            t.set("old_value", attribute_to_lua(lua, old_value.as_ref())?)?;
            t.set("new_value", attribute_to_lua(lua, new_value.as_ref())?)?;
        }
    }
    Ok(t)
}

fn set_item(lua: &Lua, event: &Table, item: &Item) -> mlua::Result<()> {
    event.set("item_id", i64::from(item.item_id))?;
    event.set("item", item_to_lua(lua, item)?)
}

fn attribute_to_lua(lua: &Lua, attr: Option<&Attribute>) -> mlua::Result<LuaValue> {
    match attr {
        None => Ok(LuaValue::Nil),
        Some(attr) => {
            let v =
                serde_json::to_value(attr).map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
            json_to_lua(lua, v)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use chrono::NaiveDate;
    use zealot_domain::{
        attribute::AttributeScalar, comment::Comment, item::ItemLink, item_type::ItemTypeRef,
    };

    use super::*;
    use crate::sandbox::{execute_script, new_sandbox};

    fn id(n: i64) -> Id {
        Id::try_from(n).unwrap()
    }

    fn sample_item() -> Item {
        let mut attributes = HashMap::new();
        attributes.insert(
            "Status".to_string(),
            Attribute::Scalar(AttributeScalar::Dropdown("Complete".to_string())),
        );
        attributes.insert(
            "Priority".to_string(),
            Attribute::Scalar(AttributeScalar::Integer(8)),
        );
        Item {
            item_id: id(42),
            title: "Write docs".to_string(),
            content: "body".to_string(),
            attributes,
            types: vec![ItemTypeRef {
                type_id: id(9),
                is_system: false,
                name: "Ticket".to_string(),
            }],
            links: vec![ItemLink {
                other_item_id: id(7),
                relationship: "parent".to_string(),
            }],
        }
    }

    /// Runs `script` as a rule body for `event` and returns its notify lines.
    async fn run(event: ZealotEvent, script: &str) -> Vec<String> {
        let lua = new_sandbox().unwrap();
        let zealot = lua.create_table().unwrap();
        let buf = Arc::new(Mutex::new(vec![]));
        register(
            &zealot,
            &lua,
            &RuleContext::Event(event),
            id(1),
            buf.clone(),
        )
        .unwrap();
        lua.globals().set("zealot", zealot).unwrap();
        execute_script(lua, script.to_string()).await.unwrap();
        let lines = buf.lock().unwrap().clone();
        lines
    }

    /// Fields every item-carrying event documents on `zealot.event.item`.
    const ITEM_FIELDS: &str = r#"
        local i = zealot.event.item
        zealot.notify(table.concat({
            zealot.event.item_id, i.id, i.title, i.content,
            i.attributes.Status, i.attributes.Priority,
            i.types[1], i.links[1].id, i.links[1].relationship,
        }, "|"))
    "#;
    const ITEM_LINE: &str = "42|42|Write docs|body|Complete|8|Ticket|7|parent";

    #[tokio::test]
    async fn item_create_and_update_provide_item() {
        for make in [
            (|item| ZealotEvent::ItemCreated {
                account_id: id(1),
                item,
            }) as fn(Item) -> ZealotEvent,
            |item| ZealotEvent::ItemUpdated {
                account_id: id(1),
                item,
            },
        ] {
            let out = run(make(sample_item()), ITEM_FIELDS).await;
            assert_eq!(out, vec![ITEM_LINE]);
            let out = run(
                make(sample_item()),
                "zealot.notify(zealot.event.kind .. ':' .. zealot.event.title)",
            )
            .await;
            assert!(out[0].ends_with(":Write docs"));
        }
    }

    #[tokio::test]
    async fn type_assign_and_unassign_provide_item_and_type_name() {
        for make in [
            (|item, type_name| ZealotEvent::TypeAssigned {
                account_id: id(1),
                item,
                type_name,
            }) as fn(Item, String) -> ZealotEvent,
            |item, type_name| ZealotEvent::TypeUnassigned {
                account_id: id(1),
                item,
                type_name,
            },
        ] {
            let out = run(make(sample_item(), "Ticket".into()), ITEM_FIELDS).await;
            assert_eq!(out, vec![ITEM_LINE]);
            let out = run(
                make(sample_item(), "Ticket".into()),
                "zealot.notify(zealot.event.type_name)",
            )
            .await;
            assert_eq!(out, vec!["Ticket"]);
        }
    }

    #[tokio::test]
    async fn attribute_set_provides_item_key_and_old_new_values() {
        let event = |old: Option<Attribute>| ZealotEvent::AttributeSet {
            account_id: id(1),
            item: sample_item(),
            attribute_key: "Status".to_string(),
            old_value: old,
            new_value: Some(Attribute::Scalar(AttributeScalar::Dropdown(
                "Complete".into(),
            ))),
        };
        let out = run(event(None), ITEM_FIELDS).await;
        assert_eq!(out, vec![ITEM_LINE]);

        let script = r#"
            local e = zealot.event
            zealot.notify(e.kind .. "|" .. e.attribute_key .. "|" ..
                tostring(e.old_value) .. "|" .. tostring(e.new_value))
        "#;
        let old = Attribute::Scalar(AttributeScalar::Dropdown("To Do".into()));
        assert_eq!(
            run(event(Some(old)), script).await,
            vec!["on_attribute_set|Status|To Do|Complete"]
        );
        assert_eq!(
            run(event(None), script).await,
            vec!["on_attribute_set|Status|nil|Complete"]
        );
    }

    #[tokio::test]
    async fn attribute_set_unset_new_value_is_nil() {
        let event = ZealotEvent::AttributeSet {
            account_id: id(1),
            item: sample_item(),
            attribute_key: "Status".to_string(),
            old_value: Some(Attribute::Scalar(AttributeScalar::Date(
                NaiveDate::from_ymd_opt(2026, 1, 2).unwrap(),
            ))),
            new_value: None,
        };
        let out = run(
            event,
            "zealot.notify(tostring(zealot.event.old_value) .. '|' .. tostring(zealot.event.new_value))",
        )
        .await;
        assert_eq!(out, vec!["2026-01-02|nil"]);
    }

    #[tokio::test]
    async fn item_delete_provides_item_id_only() {
        let event = ZealotEvent::ItemDeleted {
            account_id: id(1),
            item_id: id(42),
        };
        let out = run(
            event,
            "zealot.notify(zealot.event.kind .. '|' .. zealot.event.item_id .. '|' .. tostring(zealot.event.item))",
        )
        .await;
        assert_eq!(out, vec!["on_item_delete|42|nil"]);
    }

    #[tokio::test]
    async fn comment_add_provides_comment_fields() {
        let event = ZealotEvent::CommentAdded {
            account_id: id(1),
            item_id: id(42),
            comment: Comment {
                comment_id: id(5),
                item: sample_item(),
                timestamp: NaiveDate::from_ymd_opt(2026, 1, 2)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap(),
                content: "hello".to_string(),
            },
        };
        let out = run(
            event,
            "local e = zealot.event; zealot.notify(table.concat({e.kind, e.item_id, e.comment_id, e.content, e.timestamp}, '|'))",
        )
        .await;
        assert_eq!(out, vec!["on_comment_add|42|5|hello|2026-01-02T00:00:00"]);
    }
}

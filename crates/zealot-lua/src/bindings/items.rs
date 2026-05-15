use std::sync::Arc;

use mlua::{Lua, Table, Value as LuaValue};
use serde_json::Value as JsonValue;
use zealot_app::services::ZealotServices;
use zealot_domain::{
    account::Account,
    attribute::AttributeFilterDto,
    common::{email::Email, id::Id},
    item::{AddItemDto, Item, UpdateItemDto},
};

pub fn register(
    zealot: &Table,
    lua: &Lua,
    services: Arc<ZealotServices>,
    account_id: Id,
) -> mlua::Result<()> {
    let items_table = lua.create_table()?;

    // zealot.items.get(id) -> item or nil
    {
        let svc = services.clone();
        let acct_id = account_id;
        items_table.set(
            "get",
            lua.create_async_function(move |lua, id: i64| {
                let svc = svc.clone();
                let acct_id = acct_id;
                async move {
                    let account = make_account(acct_id)?;
                    let item_id = Id::try_from(id)
                        .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
                    match svc.item.get_item_by_id(&item_id, &account)
                        .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?
                    {
                        Some(item) => Ok(LuaValue::Table(item_to_lua(&lua, &item)?)),
                        None => Ok(LuaValue::Nil),
                    }
                }
            })?,
        )?;
    }

    // zealot.items.get_by_title(title) -> item or nil
    {
        let svc = services.clone();
        let acct_id = account_id;
        items_table.set(
            "get_by_title",
            lua.create_async_function(move |lua, title: String| {
                let svc = svc.clone();
                let acct_id = acct_id;
                async move {
                    let account = make_account(acct_id)?;
                    let items = svc
                        .item
                        .get_items_by_title(&title, &account)
                        .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
                    match items.into_iter().next() {
                        Some(item) => Ok(LuaValue::Table(item_to_lua(&lua, &item)?)),
                        None => Ok(LuaValue::Nil),
                    }
                }
            })?,
        )?;
    }

    // zealot.items.find_by_type(type_name) -> item array
    {
        let svc = services.clone();
        let acct_id = account_id;
        items_table.set(
            "find_by_type",
            lua.create_async_function(move |lua, type_name: String| {
                let svc = svc.clone();
                let acct_id = acct_id;
                async move {
                    let account = make_account(acct_id)?;
                    let items = svc
                        .item
                        .get_items_by_type(&type_name, &account)
                        .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
                    items_vec_to_lua(&lua, &items)
                }
            })?,
        )?;
    }

    // zealot.items.search(term) -> item array
    {
        let svc = services.clone();
        let acct_id = account_id;
        items_table.set(
            "search",
            lua.create_async_function(move |lua, term: String| {
                let svc = svc.clone();
                let acct_id = acct_id;
                async move {
                    let account = make_account(acct_id)?;
                    let items = svc
                        .item
                        .search_items_by_title(&term, &account)
                        .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
                    items_vec_to_lua(&lua, &items)
                }
            })?,
        )?;
    }

    // zealot.items.filter(filters) -> item array
    // filters is array of {key, op, value, list_mode?} tables
    {
        let svc = services.clone();
        let acct_id = account_id;
        items_table.set(
            "filter",
            lua.create_async_function(move |lua, filters_table: Table| {
                let svc = svc.clone();
                let acct_id = acct_id;
                async move {
                    let account = make_account(acct_id)?;
                    let mut filter_dtos: Vec<AttributeFilterDto> = Vec::new();
                    for pair in filters_table.sequence_values::<Table>() {
                        let f = pair?;
                        let key: String = f.get("key")?;
                        let op: String = f.get("op")?;
                        let list_mode: String =
                            f.get::<String>("list_mode").unwrap_or_else(|_| "any".to_string());
                        let value: LuaValue = f.get("value")?;
                        let value_json = lua_value_to_json(value)?;
                        filter_dtos.push(AttributeFilterDto { key, op, value: value_json, list_mode });
                    }
                    let items = svc
                        .item
                        .filter_items(&filter_dtos, &account)
                        .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
                    items_vec_to_lua(&lua, &items)
                }
            })?,
        )?;
    }

    // zealot.items.create(title, content?, opts?) -> item
    {
        let svc = services.clone();
        let acct_id = account_id;
        items_table.set(
            "create",
            lua.create_async_function(move |lua, (title, content, opts): (String, Option<String>, Option<Table>)| {
                let svc = svc.clone();
                let acct_id = acct_id;
                async move {
                    let account = make_account(acct_id)?;
                    let content = content.unwrap_or_default();
                    let types = opts
                        .as_ref()
                        .and_then(|o| o.get::<Table>("types").ok())
                        .map(|t| {
                            t.sequence_values::<String>()
                                .collect::<mlua::Result<Vec<_>>>()
                        })
                        .transpose()?;
                    let dto = AddItemDto {
                        title,
                        content,
                        attributes: None,
                        types,
                        links: None,
                    };
                    match svc
                        .item
                        .add_item(&dto, &account)
                        .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?
                    {
                        Some(item) => Ok(LuaValue::Table(item_to_lua(&lua, &item)?)),
                        None => Ok(LuaValue::Nil),
                    }
                }
            })?,
        )?;
    }

    // zealot.items.update(id, opts) -> item
    {
        let svc = services.clone();
        let acct_id = account_id;
        items_table.set(
            "update",
            lua.create_async_function(move |lua, (id, opts): (i64, Table)| {
                let svc = svc.clone();
                let acct_id = acct_id;
                async move {
                    let account = make_account(acct_id)?;
                    let item_id = Id::try_from(id)
                        .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
                    let title: Option<String> = opts.get("title").ok();
                    let content: Option<String> = opts.get("content").ok();
                    let dto = UpdateItemDto {
                        item_id: id,
                        title,
                        content,
                        attributes: None,
                        links: None,
                    };
                    match svc
                        .item
                        .update_item(&item_id, &dto, &account)
                        .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?
                    {
                        Some(item) => Ok(LuaValue::Table(item_to_lua(&lua, &item)?)),
                        None => Ok(LuaValue::Nil),
                    }
                }
            })?,
        )?;
    }

    // zealot.items.set_attribute(id, key, value) -> bool
    {
        let svc = services.clone();
        let acct_id = account_id;
        items_table.set(
            "set_attribute",
            lua.create_async_function(
                move |_, (id, key, value): (i64, String, LuaValue)| {
                    let svc = svc.clone();
                    let acct_id = acct_id;
                    async move {
                        let account = make_account(acct_id)?;
                        let item_id = Id::try_from(id)
                            .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
                        let value_json = lua_value_to_json(value)?;
                        let mut map = std::collections::HashMap::new();
                        map.insert(key, value_json);
                        svc.item
                            .set_attributes(&item_id, &map, &account)
                            .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
                        Ok(true)
                    }
                },
            )?,
        )?;
    }

    // zealot.items.assign_type(id, type_name) -> bool
    {
        let svc = services.clone();
        let acct_id = account_id;
        items_table.set(
            "assign_type",
            lua.create_async_function(move |_, (id, type_name): (i64, String)| {
                let svc = svc.clone();
                let acct_id = acct_id;
                async move {
                    let account = make_account(acct_id)?;
                    let item_id = Id::try_from(id)
                        .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
                    svc.item
                        .assign_type(&type_name, &item_id, &account)
                        .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
                    Ok(true)
                }
            })?,
        )?;
    }

    // zealot.items.delete(id) -> bool
    {
        let svc = services.clone();
        let acct_id = account_id;
        items_table.set(
            "delete",
            lua.create_async_function(move |_, id: i64| {
                let svc = svc.clone();
                let acct_id = acct_id;
                async move {
                    let account = make_account(acct_id)?;
                    let item_id = Id::try_from(id)
                        .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
                    svc.item
                        .delete_item(&item_id, &account)
                        .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
                    Ok(true)
                }
            })?,
        )?;
    }

    zealot.set("items", items_table)?;
    Ok(())
}

pub fn item_to_lua(lua: &Lua, item: &Item) -> mlua::Result<Table> {
    let t = lua.create_table()?;
    t.set("id", i64::from(item.item_id))?;
    t.set("title", item.title.clone())?;
    t.set("content", item.content.clone())?;

    let attrs = lua.create_table()?;
    for (key, attr) in &item.attributes {
        let v = serde_json::to_value(attr)
            .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
        attrs.set(key.clone(), json_to_lua(lua, v)?)?;
    }
    t.set("attributes", attrs)?;

    let types = lua.create_table()?;
    for (i, type_ref) in item.types.iter().enumerate() {
        types.set(i + 1, type_ref.name.clone())?;
    }
    t.set("types", types)?;

    let links = lua.create_table()?;
    for (i, link) in item.links.iter().enumerate() {
        let lt = lua.create_table()?;
        lt.set("id", i64::from(link.other_item_id))?;
        lt.set("relationship", format!("{:?}", link.relationship).to_lowercase())?;
        links.set(i + 1, lt)?;
    }
    t.set("links", links)?;

    Ok(t)
}

fn items_vec_to_lua(lua: &Lua, items: &[Item]) -> mlua::Result<Table> {
    let t = lua.create_table()?;
    for (i, item) in items.iter().enumerate() {
        t.set(i + 1, item_to_lua(lua, item)?)?;
    }
    Ok(t)
}

pub fn make_account(account_id: Id) -> mlua::Result<Account> {
    Ok(Account {
        account_id,
        username: String::new(),
        email: Email::try_from(String::from("rule@zealot.internal"))
            .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?,
        given_name: String::new(),
        surname: String::new(),
        settings: serde_json::Value::Object(Default::default()),
        has_api_key: false,
    })
}

fn lua_value_to_json(value: LuaValue) -> mlua::Result<JsonValue> {
    match value {
        LuaValue::Nil => Ok(JsonValue::Null),
        LuaValue::Boolean(b) => Ok(JsonValue::Bool(b)),
        LuaValue::Integer(i) => Ok(JsonValue::Number(i.into())),
        LuaValue::Number(n) => {
            let num = serde_json::Number::from_f64(n)
                .ok_or_else(|| mlua::Error::RuntimeError("invalid number".to_string()))?;
            Ok(JsonValue::Number(num))
        }
        LuaValue::String(s) => Ok(JsonValue::String(s.to_str()?.to_owned())),
        _ => Err(mlua::Error::RuntimeError("unsupported value type".to_string())),
    }
}

fn json_to_lua(lua: &Lua, v: JsonValue) -> mlua::Result<LuaValue> {
    match v {
        JsonValue::Null => Ok(LuaValue::Nil),
        JsonValue::Bool(b) => Ok(LuaValue::Boolean(b)),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(LuaValue::Integer(i))
            } else {
                Ok(LuaValue::Number(n.as_f64().unwrap_or(0.0)))
            }
        }
        JsonValue::String(s) => Ok(LuaValue::String(lua.create_string(s)?)),
        JsonValue::Array(arr) => {
            let t = lua.create_table()?;
            for (i, v) in arr.into_iter().enumerate() {
                t.set(i + 1, json_to_lua(lua, v)?)?;
            }
            Ok(LuaValue::Table(t))
        }
        JsonValue::Object(obj) => {
            let t = lua.create_table()?;
            for (k, v) in obj {
                t.set(k, json_to_lua(lua, v)?)?;
            }
            Ok(LuaValue::Table(t))
        }
    }
}

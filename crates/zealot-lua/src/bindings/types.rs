use std::{collections::HashMap, sync::Arc};

use mlua::{Lua, Table, Value as LuaValue};
use serde_json::Value as JsonValue;
use zealot_app::services::ZealotServices;
use zealot_domain::{attribute::AttributeKind, common::id::Id, item_type::ItemType};

use super::items::json_to_lua;

fn rt<E: std::fmt::Display>(e: E) -> mlua::Error {
    mlua::Error::RuntimeError(e.to_string())
}

pub fn register(
    zealot: &Table,
    lua: &Lua,
    services: Arc<ZealotServices>,
    account_id: Id,
) -> mlua::Result<()> {
    let types_table = lua.create_table()?;

    // zealot.types.list() -> type array ordered by name
    {
        let svc = services.clone();
        types_table.set(
            "list",
            lua.create_function(move |lua, ()| {
                let mut types = svc.item_type.get_item_types(&account_id).map_err(rt)?;
                types.sort_by(|a, b| a.name.cmp(&b.name));
                let ctx = TypeContext::load(&svc, &account_id)?;
                let out = lua.create_table()?;
                for (i, t) in types.iter().enumerate() {
                    out.set(i + 1, ctx.type_to_lua(lua, t)?)?;
                }
                Ok(out)
            })?,
        )?;
    }

    // zealot.types.get(name) -> type or nil
    {
        let svc = services.clone();
        types_table.set(
            "get",
            lua.create_function(move |lua, name: String| {
                let Some(t) = svc
                    .item_type
                    .get_item_type_by_name(&name, &account_id)
                    .map_err(rt)?
                else {
                    return Ok(LuaValue::Nil);
                };
                let ctx = TypeContext::load(&svc, &account_id)?;
                Ok(LuaValue::Table(ctx.type_to_lua(lua, &t)?))
            })?,
        )?;
    }
    zealot.set("types", types_table)?;

    let kinds_table = lua.create_table()?;

    // zealot.attribute_kinds.get(key) -> kind or nil
    {
        let svc = services.clone();
        kinds_table.set(
            "get",
            lua.create_function(move |lua, key: String| {
                match svc
                    .attribute
                    .get_kind_by_key(&key, &account_id)
                    .map_err(rt)?
                {
                    Some(kind) => Ok(LuaValue::Table(kind_to_lua(lua, &kind)?)),
                    None => Ok(LuaValue::Nil),
                }
            })?,
        )?;
    }

    // zealot.attribute_kinds.list() -> kind array ordered by key
    {
        let svc = services.clone();
        kinds_table.set(
            "list",
            lua.create_function(move |lua, ()| {
                let mut kinds = svc.attribute.get_kinds_for_user(&account_id).map_err(rt)?;
                kinds.sort_by(|a, b| a.key.cmp(&b.key));
                let out = lua.create_table()?;
                for (i, kind) in kinds.iter().enumerate() {
                    out.set(i + 1, kind_to_lua(lua, kind)?)?;
                }
                Ok(out)
            })?,
        )?;
    }
    zealot.set("attribute_kinds", kinds_table)?;
    Ok(())
}

/// Kinds and item counts, loaded once per call so a type list is not N lookups.
struct TypeContext {
    kinds: HashMap<String, AttributeKind>,
    item_counts: HashMap<i64, i64>,
}

impl TypeContext {
    fn load(svc: &ZealotServices, account_id: &Id) -> mlua::Result<Self> {
        let kinds = svc
            .attribute
            .get_kinds_for_user(account_id)
            .map_err(rt)?
            .into_iter()
            .map(|k| (k.key.clone(), k))
            .collect();
        let item_counts = svc
            .item_type
            .get_item_type_summaries(account_id)
            .map_err(rt)?
            .into_iter()
            .map(|s| (i64::from(s.type_id), s.item_count))
            .collect();
        Ok(Self { kinds, item_counts })
    }

    fn type_to_lua(&self, lua: &Lua, t: &ItemType) -> mlua::Result<Table> {
        let out = lua.create_table()?;
        let type_id = i64::from(t.type_id);
        out.set("id", type_id)?;
        out.set("name", t.name.as_str())?;
        out.set("description", t.description.as_str())?;
        out.set("is_system", t.is_system)?;
        out.set(
            "item_count",
            self.item_counts.get(&type_id).copied().unwrap_or(0),
        )?;
        let required = lua.create_table()?;
        for (i, key) in t.required_attributes.iter().enumerate() {
            let entry = match self.kinds.get(key) {
                Some(kind) => kind_to_lua(lua, kind)?,
                None => {
                    let t = lua.create_table()?;
                    t.set("key", key.as_str())?;
                    t
                }
            };
            required.set(i + 1, entry)?;
        }
        out.set("required_attributes", required)?;
        Ok(out)
    }
}

/// `{id, key, description, is_system, base_type, config, default}`; `default` repeats `config.default`.
fn kind_to_lua(lua: &Lua, kind: &AttributeKind) -> mlua::Result<Table> {
    let JsonValue::Object(mut json) = kind.to_json() else {
        return Err(rt("attribute kind did not serialize to an object"));
    };
    let out = lua.create_table()?;
    if let Some(id) = json.remove("kind_id") {
        out.set("id", json_to_lua(lua, id)?)?;
    }
    for field in ["key", "description", "is_system", "base_type"] {
        if let Some(v) = json.remove(field) {
            out.set(field, json_to_lua(lua, v)?)?;
        }
    }
    let config = json.remove("config").unwrap_or(JsonValue::Null);
    if let Some(default) = config.get("default") {
        out.set("default", json_to_lua(lua, default.clone())?)?;
    }
    out.set("config", json_to_lua(lua, config)?)?;
    Ok(out)
}

pub mod comments;
pub mod items;
pub mod statistics;
pub mod utils;

use std::sync::{Arc, Mutex};

use mlua::Lua;
use zealot_app::{ports::rule_runner::RuleContext, services::ZealotServices};
use zealot_domain::common::id::Id;

pub fn setup_zealot_globals(
    lua: &Lua,
    context: &RuleContext,
    services: Arc<ZealotServices>,
    account_id: Id,
    output_buf: Arc<Mutex<Vec<String>>>,
) -> mlua::Result<()> {
    let zealot = lua.create_table()?;

    items::register(&zealot, lua, services.clone(), account_id)?;
    comments::register(&zealot, lua, services.clone(), account_id)?;
    statistics::register(&zealot, lua, services.clone(), account_id)?;
    utils::register(&zealot, lua, context, account_id, output_buf)?;

    lua.globals().set("zealot", zealot)?;
    Ok(())
}

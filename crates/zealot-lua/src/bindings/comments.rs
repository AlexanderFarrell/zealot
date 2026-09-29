use std::sync::Arc;

use mlua::{Lua, Table};
use zealot_app::services::ZealotServices;
use zealot_domain::{
    account::Account,
    comment::AddCommentDto,
    common::{email::Email, id::Id},
};

pub fn register(
    zealot: &Table,
    lua: &Lua,
    services: Arc<ZealotServices>,
    account_id: Id,
) -> mlua::Result<()> {
    let comments_table = lua.create_table()?;

    // zealot.comments.add(item_id, content)
    {
        let svc = services.clone();
        let acct_id = account_id;
        comments_table.set(
            "add",
            lua.create_async_function(move |_, (item_id, content): (i64, String)| {
                let svc = svc.clone();
                let acct_id = acct_id;
                async move {
                    let account = make_account(acct_id)?;
                    let dto = AddCommentDto {
                        item_id,
                        content,
                        timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    };
                    svc.comment
                        .add_comment(&dto, &account)
                        .await
                        .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
                    Ok(())
                }
            })?,
        )?;
    }

    zealot.set("comments", comments_table)?;
    Ok(())
}

fn make_account(account_id: Id) -> mlua::Result<Account> {
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

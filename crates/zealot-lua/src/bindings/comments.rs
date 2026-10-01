use std::sync::Arc;

use mlua::{Lua, Table};
use zealot_app::services::ZealotServices;

use super::clock::parse_date;
use zealot_domain::{
    account::Account,
    comment::{AddCommentDto, Comment},
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

    // zealot.comments.list(item_id) -> comment array for one item, all dates
    {
        let svc = services.clone();
        let acct_id = account_id;
        comments_table.set(
            "list",
            lua.create_async_function(move |lua, item_id: i64| {
                let svc = svc.clone();
                let acct_id = acct_id;
                async move {
                    let account = make_account(acct_id)?;
                    let item_id = Id::try_from(item_id)
                        .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
                    let comments = svc
                        .comment
                        .get_for_item(&item_id, &account)
                        .await
                        .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
                    comments_to_lua(&lua, &comments)
                }
            })?,
        )?;
    }

    // zealot.comments.for_date(date) -> comment array across all items on YYYY-MM-DD
    {
        let svc = services.clone();
        let acct_id = account_id;
        comments_table.set(
            "for_date",
            lua.create_async_function(move |lua, date: String| {
                let svc = svc.clone();
                let acct_id = acct_id;
                async move {
                    let account = make_account(acct_id)?;
                    let day = parse_date(&date)?;
                    let comments = svc
                        .comment
                        .get_for_day(&day, &account)
                        .await
                        .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
                    comments_to_lua(&lua, &comments)
                }
            })?,
        )?;
    }

    zealot.set("comments", comments_table)?;
    Ok(())
}

/// Rows are `{comment_id, item_id, title, timestamp, content}`; the timestamp is the
/// stored UTC `YYYY-MM-DD HH:MM:SS` (pass it to `zealot.local_date` for the local day).
fn comments_to_lua(lua: &Lua, comments: &[Comment]) -> mlua::Result<Table> {
    let result = lua.create_table()?;
    for (i, comment) in comments.iter().enumerate() {
        let row = lua.create_table()?;
        row.set("comment_id", i64::from(comment.comment_id))?;
        row.set("item_id", i64::from(comment.item.item_id))?;
        row.set("title", comment.item.title.clone())?;
        row.set(
            "timestamp",
            comment.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
        )?;
        row.set("content", comment.content.clone())?;
        result.set(i + 1, row)?;
    }
    Ok(result)
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

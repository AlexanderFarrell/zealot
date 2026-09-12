use std::sync::Arc;

use mlua::{Lua, Table};
use zealot_app::services::{ZealotServices, scope::ScopeAccess};
use zealot_domain::comment::AddCommentDto;

pub fn register(
    zealot: &Table,
    lua: &Lua,
    services: Arc<ZealotServices>,
    access: ScopeAccess,
) -> mlua::Result<()> {
    let comments_table = lua.create_table()?;

    // zealot.comments.add(item_id, content)
    {
        let svc = services.clone();
        let access = access.clone();
        comments_table.set(
            "add",
            lua.create_async_function(move |_, (item_id, content): (i64, String)| {
                let svc = svc.clone();
                let access = access.clone();
                async move {
                    let dto = AddCommentDto {
                        item_id,
                        content,
                        timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    };
                    svc.comment
                        .add_comment_in_scopes(&dto, &access)
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

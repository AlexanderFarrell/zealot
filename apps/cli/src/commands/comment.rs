use anyhow::{Result, bail};
use chrono::Local;
use zealot_domain::comment::{AddCommentDto, CommentDto, UpdateCommentDto};

use crate::cli::CommentCmd;
use crate::commands::item::display_title;
use crate::context::{Ctx, text_or_stdin};
use crate::output::{bold, dim, green, magenta, print_json};

pub async fn run(ctx: &Ctx, cmd: CommentCmd) -> Result<()> {
    match cmd {
        CommentCmd::Add { item, text, at } => {
            let content = text_or_stdin(&text)?;
            if content.is_empty() {
                bail!("comment text is empty");
            }
            let resolved = ctx.resolve_item(&item).await?;
            let timestamp =
                at.unwrap_or_else(|| Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
            let dto = AddCommentDto {
                item_id: resolved.item_id,
                timestamp,
                content,
            };
            let comment = ctx.client.add_comment(&dto).await?;
            if ctx.json {
                return print_json(&comment);
            }
            println!(
                "{} Comment #{} on {}",
                green("✔"),
                comment.comment_id,
                bold(&resolved.title)
            );
            Ok(())
        }
        CommentCmd::Ls { day, item } => {
            let comments = if let Some(item) = item {
                let resolved = ctx.resolve_item(&item).await?;
                ctx.client.comments_for_item(resolved.item_id).await?
            } else {
                let date = day.flatten().unwrap_or_default().0;
                ctx.client.comments_for_day(date).await?
            };
            if ctx.json {
                return print_json(&comments);
            }
            if comments.is_empty() {
                println!("{}", dim("No comments."));
                return Ok(());
            }
            for comment in &comments {
                print_comment(comment);
            }
            Ok(())
        }
        CommentCmd::Edit { comment_id, text } => {
            let dto = UpdateCommentDto {
                comment_id,
                item_id: None,
                timestamp: None,
                content: Some(text),
            };
            let comment = ctx.client.update_comment(&dto).await?;
            println!("{} Updated comment #{}", green("✔"), comment.comment_id);
            Ok(())
        }
        CommentCmd::Rm { comment_id } => {
            ctx.client.delete_comment(comment_id).await?;
            println!("{} Deleted comment #{comment_id}", green("✔"));
            Ok(())
        }
    }
}

/// One-line journaling: comment on the profile's configured journal item.
pub async fn journal(ctx: &Ctx, text: &[String]) -> Result<()> {
    let journal_ref = ctx
        .profile
        .as_ref()
        .and_then(|name| ctx.config.profiles.get(name))
        .and_then(|p| p.journal_item.clone());
    let Some(journal_ref) = journal_ref else {
        bail!(
            "no journal item configured — add `journal_item = \"<item title>\"` \
             to your profile in {}",
            ctx.config_path.display()
        );
    };

    let content = text_or_stdin(text)?;
    if content.is_empty() {
        bail!("journal text is empty");
    }

    let item: crate::args::ItemRef = journal_ref.parse().map_err(anyhow::Error::msg)?;
    let resolved = ctx.resolve_item(&item).await?;
    let dto = AddCommentDto {
        item_id: resolved.item_id,
        timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        content,
    };
    let comment = ctx.client.add_comment(&dto).await?;
    if ctx.json {
        return print_json(&comment);
    }
    println!(
        "{} Logged to {} at {}",
        green("✔"),
        bold(&resolved.title),
        dim(&comment.timestamp)
    );
    Ok(())
}

fn print_comment(comment: &CommentDto) {
    println!(
        "{} {} {}",
        dim(&comment.timestamp),
        magenta(&format!(
            "{} {}",
            display_title(&comment.item),
            dim(&format!("(#{})", comment.comment_id))
        )),
        String::new()
    );
    for line in comment.content.lines() {
        println!("    {line}");
    }
}

use anyhow::{Result, bail};
use zealot_domain::statistic::{CreateStatisticEntryDto, UpdateStatisticEntryDto};

use crate::{
    cli::StatisticCmd,
    commands::item::list_items,
    context::Ctx,
    output::{bold, dim, green, print_json},
};

pub async fn run(ctx: &Ctx, cmd: StatisticCmd) -> Result<()> {
    match cmd {
        StatisticCmd::Items { parent } => {
            let parent_id = match parent {
                Some(reference) => Some(ctx.resolve_item(&reference).await?.item_id),
                None => None,
            };
            let items = ctx.client.statistic_items(parent_id).await?;
            list_items(ctx, &items)
        }
        StatisticCmd::Entries {
            item,
            start,
            end,
            limit,
            offset,
        } => {
            let item = ctx.resolve_item(&item).await?;
            let page = ctx
                .client
                .statistic_entries(
                    item.item_id,
                    start.as_deref(),
                    end.as_deref(),
                    limit,
                    offset,
                )
                .await?;
            if ctx.json {
                return print_json(&page);
            }
            if page.entries.is_empty() {
                println!("{}", dim("No Statistic Entries."));
                return Ok(());
            }
            for entry in page.entries {
                println!(
                    "{}  {}  {}{}",
                    dim(&format!("#{}", entry.statistic_entry_id)),
                    entry.occurred_at,
                    bold(&entry.value.to_string()),
                    entry
                        .comment
                        .map(|comment| format!("  — {comment}"))
                        .unwrap_or_default(),
                );
            }
            if let Some(next) = page.next_offset {
                println!(
                    "{}",
                    dim(&format!("More entries available; use --offset {next}"))
                );
            }
            Ok(())
        }
        StatisticCmd::Daily { item, start, end } => {
            let item = ctx.resolve_item(&item).await?;
            let points = ctx
                .client
                .statistic_daily(item.item_id, start.as_deref(), end.as_deref())
                .await?;
            if ctx.json {
                return print_json(&points);
            }
            if points.is_empty() {
                println!("{}", dim("No daily Statistic series."));
                return Ok(());
            }
            for point in points {
                println!(
                    "{}  {}  {}",
                    point.date,
                    bold(&point.value.to_string()),
                    dim(&format!("{} entries", point.count)),
                );
            }
            Ok(())
        }
        StatisticCmd::Summary { item, start, end } => {
            let item = ctx.resolve_item(&item).await?;
            let summary = ctx
                .client
                .statistic_summary(item.item_id, start.as_deref(), end.as_deref())
                .await?;
            if ctx.json {
                return print_json(&summary);
            }
            println!(
                "{}",
                bold(&format!("{} — {} entries", item.title, summary.count))
            );
            if summary.count == 0 {
                println!("{}", dim("Empty period."));
                return Ok(());
            }
            println!(
                "first: {}  latest: {}  min: {}  max: {}  average: {}  sum: {}  delta: {}",
                summary
                    .first
                    .map(|point| point.value.to_string())
                    .unwrap_or_else(|| "—".into()),
                summary
                    .latest
                    .map(|point| point.value.to_string())
                    .unwrap_or_else(|| "—".into()),
                display(summary.minimum),
                display(summary.maximum),
                display(summary.average),
                display(summary.sum),
                display(summary.delta),
            );
            Ok(())
        }
        StatisticCmd::Record {
            item,
            value,
            at,
            related,
            comment,
        } => {
            let item = ctx.resolve_item(&item).await?;
            let related_item_id = match related {
                Some(reference) => Some(ctx.resolve_item(&reference).await?.item_id),
                None => None,
            };
            let entry = ctx
                .client
                .create_statistic_entry(
                    item.item_id,
                    &CreateStatisticEntryDto {
                        value,
                        occurred_at: at,
                        related_item_id,
                        comment,
                    },
                )
                .await?;
            if ctx.json {
                return print_json(&entry);
            }
            println!(
                "{} Recorded {} for {} at {}",
                green("✔"),
                bold(&entry.value.to_string()),
                bold(&item.title),
                entry.occurred_at,
            );
            Ok(())
        }
        StatisticCmd::Edit {
            statistic_entry_id,
            value,
            at,
            related,
            clear_related,
            comment,
            clear_comment,
        } => {
            if value.is_none()
                && at.is_none()
                && related.is_none()
                && !clear_related
                && comment.is_none()
                && !clear_comment
            {
                bail!("provide at least one field to edit");
            }
            let related_item_id = if clear_related {
                Some(None)
            } else if let Some(reference) = related {
                Some(Some(ctx.resolve_item(&reference).await?.item_id))
            } else {
                None
            };
            let comment = if clear_comment {
                Some(None)
            } else {
                comment.map(Some)
            };
            let entry = ctx
                .client
                .update_statistic_entry(
                    statistic_entry_id,
                    &UpdateStatisticEntryDto {
                        value,
                        occurred_at: at,
                        related_item_id,
                        comment,
                    },
                )
                .await?;
            if ctx.json {
                return print_json(&entry);
            }
            println!(
                "{} Updated Statistic Entry #{}",
                green("✔"),
                entry.statistic_entry_id
            );
            Ok(())
        }
        StatisticCmd::Rm { statistic_entry_id } => {
            ctx.client
                .delete_statistic_entry(statistic_entry_id)
                .await?;
            println!(
                "{} Deleted Statistic Entry #{statistic_entry_id}",
                green("✔")
            );
            Ok(())
        }
    }
}

fn display(value: Option<f64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "—".into())
}

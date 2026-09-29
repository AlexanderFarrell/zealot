use anyhow::{Result, bail};
use chrono::{Datelike, Duration, Local, NaiveDate};
use zealot_domain::repeat::{RepeatEntryDto, UpdateRepeatEntryDto};
use zealot_domain::time_block::{CreateTimeBlockDto, TimeBlockDto, UpdateTimeBlockDto};

use crate::args::{DateArg, format_clock, iso_week_string};
use crate::cli::{BlockCmd, HabitCmd};
use crate::commands::item::{display_title, list_items};
use crate::context::Ctx;
use crate::output::{bold, cyan, dim, green, magenta, paint, print_json, yellow, zscript};

/// The day dashboard: planner items, habits, time blocks, and journal comments
/// in one view.
pub async fn day(ctx: &Ctx, date: DateArg, full: bool) -> Result<()> {
    let date = date.0;
    let (plan, habits, blocks, comments) = tokio::join!(
        ctx.client.planner_day(date),
        ctx.client.repeats_for_day(date),
        ctx.client.time_blocks_for_day(date),
        ctx.client.comments_for_day(date),
    );
    let plan = plan?;
    let habits = habits?;
    let blocks = blocks?;
    let comments = comments?;

    if ctx.json {
        return print_json(&serde_json::json!({
            "date": date.format("%Y-%m-%d").to_string(),
            "plan": plan,
            "habits": habits,
            "time_blocks": blocks,
            "comments": comments,
        }));
    }

    let today = Local::now().date_naive();
    let day_label = if date == today {
        format!("{} (today)", date.format("%A %Y-%m-%d"))
    } else {
        date.format("%A %Y-%m-%d").to_string()
    };
    println!("{}", bold(&paint("4;36", &day_label.to_uppercase())));

    if !blocks.is_empty() {
        println!("\n{}", bold("Time blocks"));
        for block in &blocks {
            println!("  {}", block_line(block));
        }
    }

    println!("\n{}", bold("Plan"));
    if plan.is_empty() {
        println!("  {}", dim("nothing planned"));
    }
    for item in &plan {
        println!(
            "  {} {}",
            cyan(&format!("#{}", item.item_id)),
            display_title(item)
        );
        if full && !item.content.trim().is_empty() {
            for line in zscript::render(&item.content).lines() {
                println!("    {line}");
            }
        }
    }

    if !habits.is_empty() {
        println!("\n{}", bold("Habits"));
        for entry in &habits {
            println!("  {}", habit_line(entry));
        }
    }

    if !comments.is_empty() {
        println!("\n{}", bold("Journal"));
        for comment in &comments {
            let time = comment
                .timestamp
                .get(11..16)
                .unwrap_or(&comment.timestamp)
                .to_string();
            println!(
                "  {} {} {}",
                dim(&time),
                magenta(&display_title(&comment.item)),
                comment.content.replace('\n', " ")
            );
        }
    }

    Ok(())
}

pub async fn week(ctx: &Ctx, week: Option<String>) -> Result<()> {
    let week = week.unwrap_or_else(|| iso_week_string(Local::now().date_naive()));
    let items = ctx.client.planner_week(&week).await?;
    if !ctx.json {
        println!("{}\n", bold(&format!("Week {week}")));
    }
    list_items(ctx, &items)
}

pub async fn month(ctx: &Ctx, month: Option<u32>, year: Option<i32>) -> Result<()> {
    let now = Local::now().date_naive();
    let month = month.unwrap_or(now.month());
    let year = year.unwrap_or(now.year());
    if !(1..=12).contains(&month) {
        bail!("month must be 1-12");
    }
    let items = ctx.client.planner_month(month, year).await?;
    if !ctx.json {
        println!("{}\n", bold(&format!("{year}-{month:02}")));
    }
    list_items(ctx, &items)
}

pub async fn year(ctx: &Ctx, year: Option<i32>) -> Result<()> {
    let year = year.unwrap_or_else(|| Local::now().year());
    let items = ctx.client.planner_year(year).await?;
    if !ctx.json {
        println!("{}\n", bold(&year.to_string()));
    }
    list_items(ctx, &items)
}

// ─── Habits ──────────────────────────────────────────────────────────────────

pub async fn habit(ctx: &Ctx, cmd: HabitCmd) -> Result<()> {
    match cmd {
        HabitCmd::Ls { date } => {
            let date = date.unwrap_or_default().0;
            let entries = ctx.client.repeats_for_day(date).await?;
            if ctx.json {
                return print_json(&entries);
            }
            if entries.is_empty() {
                println!("{}", dim("No habits for this day."));
                return Ok(());
            }
            println!("{}", bold(&date.format("%A %Y-%m-%d").to_string()));
            for entry in &entries {
                println!("  {}", habit_line(entry));
            }
            let done = entries.iter().filter(|e| e.status == "Complete").count();
            println!("{}", dim(&format!("{done}/{} complete", entries.len())));
            Ok(())
        }
        HabitCmd::Done {
            item,
            date,
            comment,
        } => set_status(ctx, item, date, Some("Complete"), comment).await,
        HabitCmd::Skip {
            item,
            date,
            comment,
        } => set_status(ctx, item, date, Some("Skip"), comment).await,
        HabitCmd::Alt {
            item,
            date,
            comment,
        } => set_status(ctx, item, date, Some("Alternate"), comment).await,
        HabitCmd::Undo { item, date } => {
            set_status(ctx, item, date, Some("Not Complete"), None).await
        }
        HabitCmd::Week { date } => habit_week(ctx, date.unwrap_or_default().0).await,
        HabitCmd::Items => {
            let items = ctx.client.repeat_items().await?;
            list_items(ctx, &items)
        }
    }
}

async fn set_status(
    ctx: &Ctx,
    item: crate::args::ItemRef,
    date: Option<DateArg>,
    status: Option<&str>,
    comment: Option<String>,
) -> Result<()> {
    let resolved = ctx.resolve_item(&item).await?;
    let date = date.unwrap_or_default().0;
    let dto = UpdateRepeatEntryDto {
        item_id: resolved.item_id,
        date: date.format("%Y-%m-%d").to_string(),
        status: status.map(String::from),
        comment,
    };
    ctx.client.set_repeat_status(&dto).await?;
    let glyph = status_glyph(status.unwrap_or("Not Complete"));
    println!(
        "{glyph} {} — {} on {}",
        bold(&resolved.title),
        status.unwrap_or("Not Complete"),
        date.format("%Y-%m-%d")
    );
    Ok(())
}

/// A 7-day grid ending at `end`: one row per habit, one column per day.
async fn habit_week(ctx: &Ctx, end: NaiveDate) -> Result<()> {
    let start = end - Duration::days(6);
    let entries = ctx.client.repeats_for_range(start, end).await?;
    if ctx.json {
        return print_json(&entries);
    }
    if entries.is_empty() {
        println!("{}", dim("No habit entries in this range."));
        return Ok(());
    }

    // Group by item title, keep stable order of first appearance.
    let mut order: Vec<String> = Vec::new();
    let mut by_title: std::collections::HashMap<String, Vec<&RepeatEntryDto>> =
        std::collections::HashMap::new();
    for entry in &entries {
        let title = entry.item.title.clone();
        if !order.contains(&title) {
            order.push(title.clone());
        }
        by_title.entry(title).or_default().push(entry);
    }

    let days: Vec<NaiveDate> = (0..7).map(|i| start + Duration::days(i)).collect();
    let title_width = order.iter().map(String::len).max().unwrap_or(0).min(30);

    // Header: weekday initials.
    let mut header = format!("{:width$}", "", width = title_width + 2);
    for day in &days {
        header.push_str(&format!(
            "{:>3}",
            day.format("%a").to_string().chars().next().unwrap_or('?')
        ));
    }
    println!("{}", dim(&header));

    for title in &order {
        let mut line = format!("{:width$}  ", truncate(title, 30), width = title_width);
        for day in &days {
            let date_str = day.format("%Y-%m-%d").to_string();
            let glyph = by_title[title]
                .iter()
                .find(|e| e.date == date_str)
                .map(|e| status_glyph(&e.status))
                .unwrap_or_else(|| dim("·"));
            line.push_str(&format!("  {glyph}"));
        }
        println!("{line}");
    }
    println!(
        "{}",
        dim(&format!(
            "{} → {}   {} complete   legend: ✔ done  ↷ skip  ◆ alternate  · none",
            start.format("%b %d"),
            end.format("%b %d"),
            entries.iter().filter(|e| e.status == "Complete").count()
        ))
    );
    Ok(())
}

fn habit_line(entry: &RepeatEntryDto) -> String {
    let glyph = status_glyph(&entry.status);
    let comment = if entry.comment.is_empty() {
        String::new()
    } else {
        dim(&format!("  — {}", entry.comment))
    };
    format!("{glyph} {}{comment}", display_title(&entry.item))
}

fn status_glyph(status: &str) -> String {
    match status {
        "Complete" => green("✔"),
        "Skip" => yellow("↷"),
        "Alternate" => cyan("◆"),
        _ => dim("·"),
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let cut: String = s.chars().take(max - 1).collect();
        format!("{cut}…")
    }
}

// ─── Time blocks ─────────────────────────────────────────────────────────────

pub async fn block(ctx: &Ctx, cmd: BlockCmd) -> Result<()> {
    match cmd {
        BlockCmd::Ls { date } => {
            let date = date.unwrap_or_default().0;
            let blocks = ctx.client.time_blocks_for_day(date).await?;
            if ctx.json {
                return print_json(&blocks);
            }
            if blocks.is_empty() {
                println!("{}", dim("No time blocks."));
                return Ok(());
            }
            println!("{}", bold(&date.format("%A %Y-%m-%d").to_string()));
            for block in &blocks {
                println!("  {}", block_line(block));
            }
            Ok(())
        }
        BlockCmd::Add {
            item,
            time,
            date,
            note,
        } => {
            let resolved = ctx.resolve_item(&item).await?;
            let date = date.unwrap_or_default().0;
            let dto = CreateTimeBlockDto {
                item_id: resolved.item_id,
                date: date.format("%Y-%m-%d").to_string(),
                start_min: time.start_min,
                end_min: time.end_min,
                note,
            };
            let block = ctx.client.create_time_block(&dto).await?;
            if ctx.json {
                return print_json(&block);
            }
            println!("{} {}", green("✔"), block_line(&block));
            Ok(())
        }
        BlockCmd::Edit {
            block_id,
            time,
            date,
            note,
        } => {
            let dto = UpdateTimeBlockDto {
                block_id,
                date: date.map(|d| d.0.format("%Y-%m-%d").to_string()),
                start_min: time.map(|t| t.start_min),
                end_min: time.map(|t| t.end_min),
                note,
            };
            ctx.client.update_time_block(&dto).await?;
            println!("{} Updated block #{block_id}", green("✔"));
            Ok(())
        }
        BlockCmd::Rm { block_id } => {
            ctx.client.delete_time_block(block_id).await?;
            println!("{} Deleted block #{block_id}", green("✔"));
            Ok(())
        }
    }
}

fn block_line(block: &TimeBlockDto) -> String {
    let note = if block.note.is_empty() {
        String::new()
    } else {
        dim(&format!("  — {}", block.note))
    };
    format!(
        "{} {}  {}{note}",
        yellow(&format!(
            "{:>5}–{:<5}",
            format_clock(block.start_min),
            format_clock(block.end_min)
        )),
        dim(&format!("[#{}]", block.block_id)),
        bold(&display_title(&block.item)),
    )
}

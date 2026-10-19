//! `zealot views`: run, inspect, and validate Saved Views.

use anyhow::{Context, Result};
use serde_json::Value;
use zealot_domain::{item::ItemDto, saved_view::ViewResultDto};

use crate::{
    cli::ViewCmd,
    commands::item::display_title,
    context::{Ctx, text_or_stdin},
    output::{bold, cyan, dim, print_json, table::Table, yellow},
};

pub async fn run(ctx: &Ctx, cmd: ViewCmd) -> Result<()> {
    match cmd {
        ViewCmd::Run {
            view,
            limit,
            offset,
        } => {
            let view = ctx.resolve_item(&view).await?;
            let result = ctx.client.run_view(view.item_id, limit, offset).await?;
            if ctx.json {
                return print_json(&result);
            }
            print_result(&result);
            Ok(())
        }
        ViewCmd::Inspect { view } => {
            let view = ctx.resolve_item(&view).await?;
            let report = ctx.client.inspect_view(view.item_id).await?;
            print_json(&report)
        }
        ViewCmd::Validate { definition } => {
            let words: Vec<String> = definition.into_iter().collect();
            let text = text_or_stdin(&words)?;
            let definition: Value =
                serde_json::from_str(&text).context("the view definition is not valid JSON")?;
            let report = ctx.client.validate_view(definition).await?;
            print_json(&report)
        }
    }
}

/// Print a view run as a table, or one table per group when the view groups its results.
fn print_result(result: &ViewResultDto) {
    println!(
        "{} {}",
        bold(&result.title),
        dim(&format!("({} of {})", result.items.len(), result.total))
    );
    for warning in &result.warnings {
        println!("{}", yellow(&format!("warning: {warning}")));
    }
    if result.items.is_empty() {
        println!("{}", dim("No items."));
        return;
    }
    if result.groups.is_empty() {
        print_table(result, result.items.iter().collect());
    } else {
        for group in &result.groups {
            println!("\n{}", bold(&group.label));
            let items = group
                .item_ids
                .iter()
                .filter_map(|id| result.items.iter().find(|i| i.item_id == *id))
                .collect();
            print_table(result, items);
        }
    }
    let shown = result.offset + result.items.len() as i64;
    if shown < result.total {
        println!("{}", dim(&format!("More: --offset {shown}")));
    }
}

/// ID, title, then one column per view column.
fn print_table(result: &ViewResultDto, items: Vec<&ItemDto>) {
    let mut headers = vec!["ID", "Title"];
    headers.extend(result.columns.iter().map(String::as_str));
    let mut table = Table::new(&headers);
    for item in items {
        let mut row = vec![cyan(&item.item_id.to_string()), display_title(item)];
        row.extend(
            result
                .columns
                .iter()
                .map(|key| match item.attributes.get(key) {
                    None | Some(Value::Null) => String::new(),
                    Some(Value::String(text)) => text.clone(),
                    Some(other) => other.to_string(),
                }),
        );
        table.row(row);
    }
    table.print();
}

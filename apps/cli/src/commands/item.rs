use std::collections::HashMap;

use anyhow::{Context, Result, bail};
use serde_json::Value;
use zealot_domain::attribute::AttributeFilterDto;
use zealot_domain::item::{AddItemDto, ItemDto, SearchScope, UpdateItemDto};

use crate::args::{ItemRef, parse_attr_pair};
use crate::cli::{ItemAttrCmd, ItemCmd, ItemTypeAssignCmd, NewArgs, ViewArgs};
use crate::context::{Ctx, read_stdin};
use crate::output::{self, bold, cyan, dim, green, print_json, table::Table, yellow, zscript};
use crate::{editor, frontmatter};

pub async fn run(ctx: Ctx, cmd: ItemCmd) -> Result<()> {
    match cmd {
        ItemCmd::View(args) => view(&ctx, args).await,
        ItemCmd::New(args) => new_item(&ctx, args).await,
        ItemCmd::Edit { item, full } => edit(&ctx, item, full).await,
        ItemCmd::Append { item, text } => append(&ctx, item, &text).await,
        ItemCmd::Rm { item, yes } => remove(&ctx, item, yes).await,
        ItemCmd::Ls { item_type } => {
            let items = ctx.client.list_items(item_type.as_deref()).await?;
            list_items(&ctx, &items)
        }
        ItemCmd::Recent { limit } => {
            let items = ctx.client.recent_items(limit, 0).await?;
            list_items(&ctx, &items)
        }
        ItemCmd::Random { count } => {
            let items = ctx.client.random_items(count).await?;
            list_items(&ctx, &items)
        }
        ItemCmd::Top { limit } => {
            let items = ctx.client.most_viewed(limit).await?;
            if ctx.json {
                return print_json(&items);
            }
            let mut table = Table::new(&["ID", "Views", "Title"]);
            for item in &items {
                table.row(vec![
                    item.item_id.to_string(),
                    item.view_count.to_string(),
                    item.title.clone(),
                ]);
            }
            table.print();
            Ok(())
        }
        ItemCmd::Children { item } => {
            let id = ctx.resolve_item(&item).await?.item_id;
            let items = ctx.client.get_children(id).await?;
            list_items(&ctx, &items)
        }
        ItemCmd::Related { item } => {
            let id = ctx.resolve_item(&item).await?.item_id;
            let items = ctx.client.get_related(id).await?;
            list_items(&ctx, &items)
        }
        ItemCmd::Backlinks { item } => {
            let id = ctx.resolve_item(&item).await?.item_id;
            let items = ctx.client.get_backlinks(id).await?;
            list_items(&ctx, &items)
        }
        ItemCmd::Attr(cmd) => attr(&ctx, cmd).await,
        ItemCmd::Type(cmd) => assign_type(&ctx, cmd).await,
        ItemCmd::Export {
            item,
            pdf,
            docx,
            out,
        } => export(&ctx, item, pdf, docx, out).await,
        ItemCmd::RebuildLinks => {
            let result = ctx.client.rebuild_links().await?;
            if ctx.json {
                return print_json(&result);
            }
            println!(
                "{} Rebuilt {} attribute links and {} wiki links.",
                green("✔"),
                result.rebuilt,
                result.wiki_rebuilt
            );
            Ok(())
        }
    }
}

pub async fn view(ctx: &Ctx, args: ViewArgs) -> Result<()> {
    let item = ctx.resolve_item(&args.item).await?;
    if ctx.json {
        return print_json(&item);
    }
    if args.raw {
        println!("{}", item.content);
        return Ok(());
    }
    if args.attrs {
        print_attributes(&item);
        return Ok(());
    }

    print_item_header(&item);
    if args.meta {
        print_attributes(&item);
        if !item.links.is_empty() {
            let links: Vec<String> = item
                .links
                .iter()
                .map(|l| format!("{}→#{}", l.relationship, l.other_item_id))
                .collect();
            println!("{}  {}", bold("Links"), dim(&links.join("  ")));
        }
        println!();
    }
    if !item.content.trim().is_empty() {
        println!("{}", zscript::render(&item.content));
    } else {
        println!("{}", dim("(no content)"));
    }
    Ok(())
}

pub async fn new_item(ctx: &Ctx, args: NewArgs) -> Result<()> {
    let mut content = match &args.content {
        Some(text) => text.clone(),
        None if args.edit => String::new(),
        None => {
            use std::io::IsTerminal;
            if std::io::stdin().is_terminal() {
                String::new()
            } else {
                read_stdin()?
            }
        }
    };
    if args.edit {
        if let Some(edited) = editor::edit_text(&content, "md")? {
            content = edited;
        }
    }

    let mut attributes: HashMap<String, Value> = HashMap::new();
    for pair in &args.attrs {
        let (key, value) = parse_attr_pair(pair).map_err(anyhow::Error::msg)?;
        attributes.insert(key, value);
    }
    if let Some(parent) = &args.parent {
        let parent = ctx.resolve_item(parent).await?;
        // Parent is an item-typed attribute; the link index derives from it.
        attributes.insert("Parent".to_string(), Value::from(parent.item_id));
    }

    let dto = AddItemDto {
        title: args.title.clone(),
        content,
        attributes: (!attributes.is_empty()).then_some(attributes),
        types: (!args.types.is_empty()).then_some(args.types.clone()),
        links: None,
    };
    let item = ctx.client.add_item(&dto).await?;
    if ctx.json {
        return print_json(&item);
    }
    println!(
        "{} Created {} {}",
        green("✔"),
        cyan(&format!("#{}", item.item_id)),
        bold(&item.title)
    );
    Ok(())
}

async fn edit(ctx: &Ctx, item_ref: ItemRef, full: bool) -> Result<()> {
    let item = ctx.resolve_item(&item_ref).await?;

    if full {
        return edit_full(ctx, item).await;
    }

    let Some(content) = editor::edit_text(&item.content, "md")? else {
        println!("{}", dim("No changes."));
        return Ok(());
    };
    let dto = UpdateItemDto {
        item_id: item.item_id,
        title: None,
        content: Some(content),
        attributes: None,
        links: None,
    };
    let updated = ctx.client.update_item(&dto).await?;
    println!(
        "{} Updated {} {}",
        green("✔"),
        cyan(&format!("#{}", updated.item_id)),
        bold(&updated.title)
    );
    Ok(())
}

/// Frontmatter editing: title, types, and attributes alongside the content.
/// On frontmatter errors the buffer is reopened with the error as a comment
/// so the edit is never lost.
async fn edit_full(ctx: &Ctx, item: ItemDto) -> Result<()> {
    let mut buffer = frontmatter::to_buffer(&item);
    let parsed = loop {
        let Some(edited) = editor::edit_text(&buffer, "toml")? else {
            println!("{}", dim("No changes."));
            return Ok(());
        };
        match frontmatter::parse_buffer(strip_error_banner(&edited)) {
            Ok(parsed) => break parsed,
            Err(e) => {
                buffer = format!("# ERROR: {e}\n# Fix the frontmatter and save again.\n{edited}");
            }
        }
    };

    let dto = UpdateItemDto {
        item_id: item.item_id,
        title: (parsed.title != item.title).then_some(parsed.title.clone()),
        content: (parsed.content != item.content).then_some(parsed.content.clone()),
        attributes: Some(parsed.attributes.clone()),
        links: None,
    };
    ctx.client.update_item(&dto).await?;

    // Attribute removals aren't expressed by the update DTO — delete explicitly.
    if let Value::Object(old_attrs) = &item.attributes {
        for key in old_attrs.keys() {
            if !parsed.attributes.contains_key(key) {
                ctx.client.delete_item_attribute(item.item_id, key).await?;
            }
        }
    }

    // Type changes: diff assigned names.
    let old_types: Vec<&str> = item.types.iter().map(|t| t.name.as_str()).collect();
    for type_name in &parsed.types {
        if !old_types.contains(&type_name.as_str()) {
            ctx.client.assign_type(item.item_id, type_name).await?;
        }
    }
    for type_name in old_types {
        if !parsed.types.iter().any(|t| t == type_name) {
            ctx.client.unassign_type(item.item_id, type_name).await?;
        }
    }

    println!(
        "{} Updated {} {}",
        green("✔"),
        cyan(&format!("#{}", item.item_id)),
        bold(&parsed.title)
    );
    Ok(())
}

fn strip_error_banner(text: &str) -> &str {
    let mut rest = text;
    while rest.starts_with("# ") {
        match rest.find('\n') {
            Some(idx) => rest = &rest[idx + 1..],
            None => return "",
        }
    }
    rest
}

async fn append(ctx: &Ctx, item_ref: ItemRef, text: &[String]) -> Result<()> {
    let addition = crate::context::text_or_stdin(text)?;
    if addition.is_empty() {
        bail!("nothing to append");
    }
    let item = ctx.resolve_item(&item_ref).await?;
    let mut content = item.content.clone();
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(&addition);
    content.push('\n');

    let dto = UpdateItemDto {
        item_id: item.item_id,
        title: None,
        content: Some(content),
        attributes: None,
        links: None,
    };
    ctx.client.update_item(&dto).await?;
    println!(
        "{} Appended to {} {}",
        green("✔"),
        cyan(&format!("#{}", item.item_id)),
        bold(&item.title)
    );
    Ok(())
}

async fn remove(ctx: &Ctx, item_ref: ItemRef, yes: bool) -> Result<()> {
    let item = ctx.resolve_item(&item_ref).await?;
    if !yes {
        use std::io::Write;
        print!(
            "Delete {} {}? [y/N] ",
            cyan(&format!("#{}", item.item_id)),
            bold(&item.title)
        );
        std::io::stdout().flush()?;
        let mut answer = String::new();
        std::io::stdin().read_line(&mut answer)?;
        if !matches!(answer.trim().to_lowercase().as_str(), "y" | "yes") {
            println!("Aborted.");
            return Ok(());
        }
    }
    ctx.client.delete_item(item.item_id).await?;
    println!("{} Deleted #{} {}", green("✔"), item.item_id, item.title);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn search(
    ctx: &Ctx,
    term: &str,
    content: bool,
    heading: bool,
    regex: bool,
    limit: i64,
    offset: i64,
    all: bool,
) -> Result<()> {
    let scope = if content {
        SearchScope::Content
    } else if heading {
        SearchScope::Heading
    } else {
        SearchScope::Title
    };

    let mut results = Vec::new();
    let mut offset = offset;
    loop {
        let page = ctx
            .client
            .search_items(term, scope.clone(), regex, limit, offset)
            .await?;
        let page_len = page.len() as i64;
        results.extend(page);
        if !all || page_len < limit {
            break;
        }
        offset += limit;
    }

    if ctx.json {
        return print_json(&results);
    }
    if results.is_empty() {
        println!("{}", dim("No matches."));
        return Ok(());
    }
    for result in &results {
        let id = cyan(&format!("#{}", result.item.item_id));
        let types = type_suffix(&result.item);
        println!("{id}  {}{types}", bold(&result.item.title));
        if let Some(snippet) = &result.snippet {
            println!("      {}", dim(&snippet.replace('\n', " ")));
        }
    }
    println!("{}", dim(&format!("{} result(s)", results.len())));
    Ok(())
}

/// Parse filter expressions like `Status=Open`, `Due<=2026-07-10`, `Note~recipe`.
pub async fn filter(ctx: &Ctx, exprs: &[String], limit: i64, offset: i64) -> Result<()> {
    let filters: Vec<AttributeFilterDto> = exprs
        .iter()
        .map(|expr| parse_filter_expr(expr))
        .collect::<Result<_>>()?;
    let items = ctx.client.filter_items(&filters, limit, offset).await?;
    list_items(ctx, &items)
}

fn parse_filter_expr(expr: &str) -> Result<AttributeFilterDto> {
    // Longest operators first so `<=` wins over `<`.
    for op_text in ["<=", ">=", "!=", "<>", "=", "<", ">", "~"] {
        if let Some(idx) = expr.find(op_text) {
            let key = expr[..idx].trim();
            let value = expr[idx + op_text.len()..].trim();
            if key.is_empty() {
                bail!("missing attribute key in filter '{expr}'");
            }
            let op = match op_text {
                "~" => "ilike".to_string(),
                other => other.to_string(),
            };
            let value = if op == "ilike" {
                // Wrap in % for substring semantics unless the user provided wildcards.
                if value.contains('%') {
                    Value::String(value.to_string())
                } else {
                    Value::String(format!("%{value}%"))
                }
            } else {
                crate::args::coerce_value(value)
            };
            return Ok(AttributeFilterDto {
                key: key.to_string(),
                op,
                value,
                list_mode: "any".to_string(),
            });
        }
    }
    bail!("invalid filter '{expr}' (expected KEY=VALUE, KEY<=VALUE, KEY~TEXT, …)")
}

async fn attr(ctx: &Ctx, cmd: ItemAttrCmd) -> Result<()> {
    match cmd {
        ItemAttrCmd::Ls { item } => {
            let item = ctx.resolve_item(&item).await?;
            if ctx.json {
                return print_json(&item.attributes);
            }
            print_item_header(&item);
            print_attributes(&item);
            Ok(())
        }
        ItemAttrCmd::Set { item, pairs } => {
            let item = ctx.resolve_item(&item).await?;
            let mut attrs = HashMap::new();
            for pair in &pairs {
                let (key, value) = parse_attr_pair(pair).map_err(anyhow::Error::msg)?;
                attrs.insert(key, value);
            }
            ctx.client.set_item_attributes(item.item_id, &attrs).await?;
            println!(
                "{} Set {} attribute(s) on #{} {}",
                green("✔"),
                attrs.len(),
                item.item_id,
                bold(&item.title)
            );
            Ok(())
        }
        ItemAttrCmd::Rm { item, key } => {
            let item = ctx.resolve_item(&item).await?;
            ctx.client.delete_item_attribute(item.item_id, &key).await?;
            println!("{} Removed '{key}' from #{}", green("✔"), item.item_id);
            Ok(())
        }
        ItemAttrCmd::Rename {
            item,
            old_key,
            new_key,
        } => {
            let item = ctx.resolve_item(&item).await?;
            ctx.client
                .rename_item_attribute(item.item_id, &old_key, &new_key)
                .await?;
            println!(
                "{} Renamed '{old_key}' → '{new_key}' on #{}",
                green("✔"),
                item.item_id
            );
            Ok(())
        }
    }
}

async fn assign_type(ctx: &Ctx, cmd: ItemTypeAssignCmd) -> Result<()> {
    match cmd {
        ItemTypeAssignCmd::Add { item, type_name } => {
            let item = ctx.resolve_item(&item).await?;
            ctx.client.assign_type(item.item_id, &type_name).await?;
            println!(
                "{} #{} {} is now a {}",
                green("✔"),
                item.item_id,
                bold(&item.title),
                yellow(&type_name)
            );
            Ok(())
        }
        ItemTypeAssignCmd::Rm { item, type_name } => {
            let item = ctx.resolve_item(&item).await?;
            ctx.client.unassign_type(item.item_id, &type_name).await?;
            println!(
                "{} Removed type {} from #{}",
                green("✔"),
                yellow(&type_name),
                item.item_id
            );
            Ok(())
        }
    }
}

async fn export(
    ctx: &Ctx,
    item_ref: ItemRef,
    pdf: bool,
    docx: bool,
    out: Option<String>,
) -> Result<()> {
    if pdf == docx {
        bail!("choose exactly one of --pdf or --docx");
    }
    let item = ctx.resolve_item(&item_ref).await?;
    let download = if pdf {
        ctx.client.export_item_pdf(item.item_id).await?
    } else {
        ctx.client.export_item_docx(item.item_id).await?
    };
    let filename = out
        .or(download.filename.clone())
        .unwrap_or_else(|| format!("{}.{}", item.title, if pdf { "pdf" } else { "docx" }));
    std::fs::write(&filename, &download.bytes)
        .with_context(|| format!("failed to write {filename}"))?;
    println!(
        "{} Exported #{} to {} ({} bytes)",
        green("✔"),
        item.item_id,
        bold(&filename),
        download.bytes.len()
    );
    Ok(())
}

// ─── Display helpers ─────────────────────────────────────────────────────────

pub fn list_items(ctx: &Ctx, items: &[ItemDto]) -> Result<()> {
    if ctx.json {
        return print_json(&items);
    }
    if items.is_empty() {
        println!("{}", dim("No items."));
        return Ok(());
    }
    let mut table = Table::new(&["ID", "Title", "Types"]);
    for item in items {
        table.row(vec![
            cyan(&item.item_id.to_string()),
            display_title(item),
            dim(&item
                .types
                .iter()
                .map(|t| t.name.clone())
                .collect::<Vec<_>>()
                .join(", ")),
        ]);
    }
    table.print();
    Ok(())
}

pub fn display_title(item: &ItemDto) -> String {
    if let Some(Value::String(icon)) = item.attributes.get("Icon") {
        format!("{icon} {}", item.title)
    } else {
        item.title.clone()
    }
}

fn type_suffix(item: &ItemDto) -> String {
    if item.types.is_empty() {
        String::new()
    } else {
        dim(&format!(
            "  ({})",
            item.types
                .iter()
                .map(|t| t.name.clone())
                .collect::<Vec<_>>()
                .join(", ")
        ))
    }
}

pub fn print_item_header(item: &ItemDto) {
    println!(
        "{}  {}{}",
        cyan(&format!("#{}", item.item_id)),
        output::bold(&display_title(item)),
        type_suffix(item)
    );
}

pub fn print_attributes(item: &ItemDto) {
    let Value::Object(attrs) = &item.attributes else {
        return;
    };
    if attrs.is_empty() {
        println!("{}", dim("(no attributes)"));
        return;
    }
    let width = attrs.keys().map(String::len).max().unwrap_or(0);
    for (key, value) in attrs {
        let rendered = match value {
            Value::String(s) => s.clone(),
            other => other.to_string(),
        };
        println!("  {}{}  {}", bold(key), " ".repeat(width - key.len()), rendered);
    }
}

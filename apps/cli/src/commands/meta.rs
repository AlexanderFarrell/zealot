//! Item type and attribute-kind management.

use anyhow::{Context, Result};
use zealot_domain::{attribute::AddAttributeKindDto, item_type::AddItemTypeDto};

use crate::cli::{AttrCmd, TypeCmd};
use crate::context::Ctx;
use crate::output::{bold, dim, green, print_json, table::Table, yellow};

pub async fn item_type(ctx: &Ctx, cmd: TypeCmd) -> Result<()> {
    match cmd {
        TypeCmd::Ls { counts } => {
            if counts {
                let summaries = ctx.client.item_type_summaries().await?;
                if ctx.json {
                    return print_json(&summaries);
                }
                let mut table = Table::new(&["Type", "Items", "Required attrs", "System"]);
                for s in &summaries {
                    table.row(vec![
                        bold(&s.name),
                        s.item_count.to_string(),
                        s.required_attributes_count.to_string(),
                        if s.is_system { dim("yes") } else { String::new() },
                    ]);
                }
                table.print();
                return Ok(());
            }
            let types = ctx.client.list_item_types().await?;
            if ctx.json {
                return print_json(&types);
            }
            let mut table = Table::new(&["Type", "Description", "Required attrs"]);
            for t in &types {
                table.row(vec![
                    bold(&t.name),
                    t.description.clone(),
                    dim(&t.required_attributes.join(", ")),
                ]);
            }
            table.print();
            Ok(())
        }
        TypeCmd::View { name } => {
            let t = ctx.client.get_item_type_by_name(&name).await?;
            if ctx.json {
                return print_json(&t);
            }
            println!("{} {}", bold(&t.name), dim(&format!("(#{})", t.type_id)));
            if !t.description.is_empty() {
                println!("{}", t.description);
            }
            if !t.required_attributes.is_empty() {
                println!(
                    "{} {}",
                    bold("Required:"),
                    t.required_attributes.join(", ")
                );
            }
            Ok(())
        }
        TypeCmd::New {
            name,
            description,
            required_attributes,
        } => {
            let dto = AddItemTypeDto {
                name: name.clone(),
                description,
                required_attributes,
            };
            let t = ctx.client.add_item_type(&dto).await?;
            if ctx.json {
                return print_json(&t);
            }
            println!("{} Created type {}", green("✔"), yellow(&t.name));
            Ok(())
        }
        TypeCmd::Rm { name, force } => {
            let t = ctx
                .client
                .get_item_type_by_name(&name)
                .await
                .with_context(|| format!("no item type named '{name}'"))?;
            ctx.client.delete_item_type(t.type_id, force).await?;
            println!("{} Deleted type {}", green("✔"), yellow(&name));
            Ok(())
        }
    }
}

pub async fn attr_kind(ctx: &Ctx, cmd: AttrCmd) -> Result<()> {
    match cmd {
        AttrCmd::Ls => {
            let kinds = ctx.client.list_attribute_kinds().await?;
            if ctx.json {
                return print_json(&kinds);
            }
            let mut table = Table::new(&["Key", "Type", "Description", "System"]);
            for kind in &kinds {
                table.row(vec![
                    bold(&kind.key),
                    yellow(&kind.base_type),
                    kind.description.clone(),
                    if kind.is_system { dim("yes") } else { String::new() },
                ]);
            }
            table.print();
            Ok(())
        }
        AttrCmd::View { key } => {
            let kind = ctx.client.get_attribute_kind_by_key(&key).await?;
            if ctx.json {
                return print_json(&kind);
            }
            println!(
                "{} {} {}",
                bold(&kind.key),
                yellow(&kind.base_type),
                dim(&format!("(#{})", kind.kind_id))
            );
            if !kind.description.is_empty() {
                println!("{}", kind.description);
            }
            if !kind.config.is_null() && kind.config != serde_json::json!({}) {
                println!("{} {}", bold("Config:"), kind.config);
            }
            Ok(())
        }
        AttrCmd::New {
            key,
            base,
            description,
            config,
        } => {
            let config: serde_json::Value =
                serde_json::from_str(&config).context("--config must be valid JSON")?;
            let dto = AddAttributeKindDto {
                key: key.clone(),
                description,
                base_type: base,
                config,
            };
            let kind = ctx.client.add_attribute_kind(&dto).await?;
            if ctx.json {
                return print_json(&kind);
            }
            println!(
                "{} Created attribute kind {} ({})",
                green("✔"),
                bold(&kind.key),
                kind.base_type
            );
            Ok(())
        }
        AttrCmd::Rm { key, force } => {
            ctx.client.delete_attribute_kind(&key, force).await?;
            println!("{} Deleted attribute kind {}", green("✔"), bold(&key));
            Ok(())
        }
    }
}

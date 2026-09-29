use anyhow::{Context, Result};
use zealot_domain::rule::{RuleDto, UpdateRuleDto};

use crate::cli::RuleCmd;
use crate::context::Ctx;
use crate::output::{bold, cyan, dim, green, print_json, red, table::Table, yellow};
use crate::editor;

pub async fn run(ctx: &Ctx, cmd: RuleCmd) -> Result<()> {
    match cmd {
        RuleCmd::Ls => {
            let rules = ctx.client.list_rules().await?;
            if ctx.json {
                return print_json(&rules);
            }
            if rules.is_empty() {
                println!("{}", dim("No rules."));
                return Ok(());
            }
            let mut table = Table::new(&["ID", "Rule", "Trigger", "On", "Last run"]);
            for rule in &rules {
                table.row(vec![
                    cyan(&rule.rule_id.to_string()),
                    bold(&rule.name),
                    yellow(rule.trigger.kind_str()),
                    if rule.enabled {
                        green("✔")
                    } else {
                        dim("off")
                    },
                    last_run_summary(rule),
                ]);
            }
            table.print();
            Ok(())
        }
        RuleCmd::View { rule, script } => {
            let rule = resolve_rule(ctx, &rule).await?;
            if ctx.json {
                return print_json(&rule);
            }
            println!(
                "{} {} {}",
                cyan(&format!("#{}", rule.rule_id)),
                bold(&rule.name),
                if rule.enabled {
                    green("enabled")
                } else {
                    dim("disabled")
                }
            );
            if !rule.description.is_empty() {
                println!("{}", rule.description);
            }
            println!(
                "{} {}",
                bold("Trigger:"),
                serde_json::to_string(&rule.trigger)?
            );
            println!("{} {}", bold("Last run:"), last_run_summary(&rule));
            if let Some(output) = &rule.last_output {
                if !output.is_empty() {
                    println!("{}\n{output}", bold("Last output:"));
                }
            }
            if let Some(error) = &rule.last_error {
                if !error.is_empty() {
                    println!("{}\n{}", bold("Last error:"), red(error));
                }
            }
            if script {
                println!("\n{}", dim("── script ──"));
                println!("{}", rule.script);
            }
            Ok(())
        }
        RuleCmd::Run { rule } => {
            let rule = resolve_rule(ctx, &rule).await?;
            let result = ctx.client.run_rule(rule.rule_id).await?;
            if ctx.json {
                return print_json(&result);
            }
            if result.success {
                println!(
                    "{} {} ran in {}ms",
                    green("✔"),
                    bold(&rule.name),
                    result.duration_ms
                );
            } else {
                println!(
                    "{} {} failed after {}ms",
                    red("✘"),
                    bold(&rule.name),
                    result.duration_ms
                );
            }
            if let Some(output) = result.output.filter(|o| !o.is_empty()) {
                println!("{output}");
            }
            if let Some(error) = result.error.filter(|e| !e.is_empty()) {
                println!("{}", red(&error));
                anyhow::bail!("rule failed");
            }
            Ok(())
        }
        RuleCmd::Enable { rule } => set_enabled(ctx, &rule, true).await,
        RuleCmd::Disable { rule } => set_enabled(ctx, &rule, false).await,
        RuleCmd::Edit { rule } => {
            let rule = resolve_rule(ctx, &rule).await?;
            let Some(script) = editor::edit_text(&rule.script, "lua")? else {
                println!("{}", dim("No changes."));
                return Ok(());
            };
            let dto = UpdateRuleDto {
                name: None,
                description: None,
                trigger: None,
                script: Some(script),
                enabled: None,
            };
            ctx.client.update_rule(rule.rule_id, &dto).await?;
            println!("{} Updated script for {}", green("✔"), bold(&rule.name));
            Ok(())
        }
    }
}

async fn set_enabled(ctx: &Ctx, rule: &str, enabled: bool) -> Result<()> {
    let rule = resolve_rule(ctx, rule).await?;
    let dto = UpdateRuleDto {
        name: None,
        description: None,
        trigger: None,
        script: None,
        enabled: Some(enabled),
    };
    ctx.client.update_rule(rule.rule_id, &dto).await?;
    println!(
        "{} {} is now {}",
        green("✔"),
        bold(&rule.name),
        if enabled { "enabled" } else { "disabled" }
    );
    Ok(())
}

/// Rules can be referenced by numeric id or by (case-insensitive) name.
async fn resolve_rule(ctx: &Ctx, reference: &str) -> Result<RuleDto> {
    if let Ok(id) = reference.parse::<i64>() {
        return ctx
            .client
            .get_rule(id)
            .await
            .with_context(|| format!("rule #{id} not found"));
    }
    let rules = ctx.client.list_rules().await?;
    let lower = reference.to_lowercase();
    rules
        .into_iter()
        .find(|r| r.name.to_lowercase() == lower)
        .with_context(|| format!("no rule named '{reference}'"))
}

fn last_run_summary(rule: &RuleDto) -> String {
    match &rule.last_run_at {
        Some(at) => {
            let when = at.format("%Y-%m-%d %H:%M").to_string();
            if rule.last_error.as_deref().is_some_and(|e| !e.is_empty()) {
                format!("{} {}", when, red("(error)"))
            } else {
                when
            }
        }
        None => dim("never"),
    }
}

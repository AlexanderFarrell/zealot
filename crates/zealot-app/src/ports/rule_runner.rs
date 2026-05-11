use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use chrono::NaiveDateTime;
use zealot_domain::{common::id::Id, rule::Rule};
use crate::ports::events::ZealotEvent;

#[derive(Debug, Clone, serde::Serialize)]
pub struct RuleRunResult {
    pub rule_id:     Id,
    pub success:     bool,
    pub output:      Option<String>,
    pub error:       Option<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone)]
pub enum RuleContext {
    Event(ZealotEvent),
    Scheduled { now: NaiveDateTime },
    Manual,
}

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait RuleRunnerPort: Debug + Send + Sync {
    /// Find and run all enabled event-triggered rules for this event.
    fn run_event_rules(&self, event: ZealotEvent) -> BoxFuture<'_, Vec<RuleRunResult>>;

    /// Run a single rule with the given context.
    fn run_rule<'a>(&'a self, rule: &'a Rule, context: RuleContext) -> BoxFuture<'a, RuleRunResult>;
}

/// No-op implementation for use in tests and contexts without Lua.
#[derive(Debug)]
pub struct NoopRuleRunner;

impl RuleRunnerPort for NoopRuleRunner {
    fn run_event_rules(&self, _event: ZealotEvent) -> BoxFuture<'_, Vec<RuleRunResult>> {
        Box::pin(async { vec![] })
    }

    fn run_rule<'a>(&'a self, rule: &'a Rule, _context: RuleContext) -> BoxFuture<'a, RuleRunResult> {
        Box::pin(async move {
            RuleRunResult {
                rule_id:     rule.rule_id,
                success:     true,
                output:      None,
                error:       None,
                duration_ms: 0,
            }
        })
    }
}

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use chrono::Utc;
use zealot_app::{
    ports::events::{ZealotEvent, with_rule_depth_guard_async},
    ports::rule_runner::{RuleContext, RuleRunResult, RuleRunnerPort},
    repos::ZealotRepos,
    services::ZealotServices,
};
use zealot_domain::rule::Rule;

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[derive(Debug, Clone)]
pub struct LuaRuleRunner {
    repos: Arc<ZealotRepos>,
    services: Arc<ZealotServices>,
}

impl LuaRuleRunner {
    pub fn new(repos: Arc<ZealotRepos>, services: Arc<ZealotServices>) -> Self {
        Self { repos, services }
    }

    async fn execute_one(&self, rule: &Rule, context: RuleContext) -> RuleRunResult {
        let start = std::time::Instant::now();
        let output_buf: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

        let result = self.run_script(rule, &context, output_buf.clone()).await;

        let duration_ms = start.elapsed().as_millis() as u64;
        let output: Option<String> = {
            let lines = output_buf.lock().unwrap();
            if lines.is_empty() {
                None
            } else {
                Some(lines.join("\n"))
            }
        };

        let (success, error) = match result {
            Ok(()) => (true, None),
            Err(e) => (false, Some(e.to_string())),
        };

        let now = Utc::now().naive_utc();
        let _ = self
            .repos
            .rule
            .record_run(&rule.rule_id, now, error.as_deref(), output.as_deref());

        RuleRunResult {
            rule_id: rule.rule_id,
            success,
            output,
            error,
            duration_ms,
        }
    }

    async fn run_script(
        &self,
        rule: &Rule,
        context: &RuleContext,
        output_buf: Arc<Mutex<Vec<String>>>,
    ) -> mlua::Result<()> {
        use crate::bindings::setup_zealot_globals;
        use crate::sandbox::{execute_script, new_sandbox};

        let access = self
            .services
            .scope
            .rule_access(rule.scope_id)
            .map_err(|error| mlua::Error::RuntimeError(error.to_string()))?
            .ok_or_else(|| mlua::Error::RuntimeError("rule scope is not active".to_string()))?;
        let lua = new_sandbox()?;
        setup_zealot_globals(
            &lua,
            context,
            self.services.clone(),
            rule.account_id,
            access,
            output_buf,
        )?;
        execute_script(lua, rule.script.clone()).await
    }

    async fn run_event_rules_inner(&self, event: ZealotEvent) -> Vec<RuleRunResult> {
        let trigger_kind = event.trigger_kind();

        let rules = match self
            .repos
            .rule
            .get_enabled_event_rules_in_scope(trigger_kind, event.scope_id())
        {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Failed to load event rules: {e}");
                return vec![];
            }
        };

        let mut results = vec![];
        for rule in rules {
            let context = RuleContext::Event(event.clone());
            let runner = self.clone();
            let result =
                with_rule_depth_guard_async(
                    || async move { runner.execute_one(&rule, context).await },
                )
                .await;
            results.push(result);
        }
        results
    }
}

impl RuleRunnerPort for LuaRuleRunner {
    fn run_event_rules(&self, event: ZealotEvent) -> BoxFuture<'_, Vec<RuleRunResult>> {
        Box::pin(self.run_event_rules_inner(event))
    }

    fn run_rule<'a>(
        &'a self,
        rule: &'a Rule,
        context: RuleContext,
    ) -> BoxFuture<'a, RuleRunResult> {
        Box::pin(self.execute_one(rule, context))
    }
}

use tokio::time::{sleep, Duration};
use chrono::Utc;
use zealot_domain::rule::TriggerKind;
use crate::{app::AppState, ports::rule_runner::RuleContext};

pub fn start(state: AppState) {
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(60)).await;
            run_scheduled_rules(&state).await;
        }
    });
}

async fn run_scheduled_rules(state: &AppState) {
    let now = Utc::now().naive_local();

    let scheduled = match state.services.rule.get_all_scheduled_rules() {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Scheduler: failed to load scheduled rules: {e}");
            return;
        }
    };

    for (rule, _account_id) in scheduled {
        if is_due(&rule.trigger, rule.last_run_at, now) {
            let runner = state.ports.rule_runner.clone();
            let r = rule.clone();
            tokio::spawn(async move {
                runner.run_rule(&r, RuleContext::Scheduled { now }).await;
            });
        }
    }
}

fn is_due(
    trigger: &TriggerKind,
    last_run_at: Option<chrono::NaiveDateTime>,
    now: chrono::NaiveDateTime,
) -> bool {
    match trigger {
        TriggerKind::Cron { expression } => {
            use croner::Cron;
            let Ok(cron) = Cron::new(expression).parse() else { return false };
            let last = last_run_at.unwrap_or(chrono::NaiveDateTime::MIN);
            let secs_since_last = (now - last).num_seconds();
            secs_since_last >= 55 && cron.is_time_matching(&now.and_utc()).unwrap_or(false)
        }
        TriggerKind::Interval { seconds } => {
            let last = last_run_at.unwrap_or(chrono::NaiveDateTime::MIN);
            (now - last).num_seconds() >= *seconds as i64
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;

    fn dt(s: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").unwrap()
    }

    #[test]
    fn interval_due_when_enough_time_elapsed() {
        let trigger = TriggerKind::Interval { seconds: 65 };
        let last = dt("2024-01-01 12:00:00");
        let now  = dt("2024-01-01 12:01:06");
        assert!(is_due(&trigger, Some(last), now));
    }

    #[test]
    fn interval_not_due_when_too_soon() {
        let trigger = TriggerKind::Interval { seconds: 65 };
        let last = dt("2024-01-01 12:00:00");
        let now  = dt("2024-01-01 12:00:30");
        assert!(!is_due(&trigger, Some(last), now));
    }

    #[test]
    fn cron_not_due_when_run_recently() {
        let trigger = TriggerKind::Cron { expression: "* * * * *".to_string() };
        let last = dt("2024-01-01 12:00:00");
        let now  = dt("2024-01-01 12:00:05");
        assert!(!is_due(&trigger, Some(last), now));
    }
}

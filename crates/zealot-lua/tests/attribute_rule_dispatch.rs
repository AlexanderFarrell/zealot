//! Z212: an `on_attribute_set` rule runs only for its own attribute and only when the value
//! changed. Drives real writes (Lua `set_attribute`, `create`, `assign_type`) against an
//! in-memory sqlite database, then dispatches each emitted event through the real rule runner.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use serde_json::json;

use sqlx::sqlite::SqlitePoolOptions;
use zealot_app::{
    ports::{
        ZealotPorts,
        events::{EventPort, ZealotEvent},
        rule_runner::{NoopRuleRunner, RuleContext, RuleRunnerPort},
    },
    repos::ZealotRepos,
    services::ZealotServices,
};
use zealot_domain::{
    common::id::Id,
    item::AddItemDto,
    rule::{AddRuleDto, TriggerKind},
};
use zealot_infra::{
    ports::{media::in_memory::MediaInMemoryPort, password::bcrypt_password::BcryptPasswordPort},
    repos::sqlite::get_sqlite_repos,
};
use zealot_lua::{
    LuaRuleRunner,
    bindings::{items::make_account, setup_zealot_globals},
    sandbox::{execute_script, new_sandbox},
};

/// Collects every emitted event so the test can dispatch them one by one.
#[derive(Debug, Default)]
struct RecordingEvents(Mutex<Vec<ZealotEvent>>);

impl EventPort for RecordingEvents {
    fn emit(&self, event: ZealotEvent) {
        self.0.lock().unwrap().push(event);
    }
}

struct Harness {
    repos: Arc<ZealotRepos>,
    services: Arc<ZealotServices>,
    runner: LuaRuleRunner,
    events: Arc<RecordingEvents>,
}

async fn harness() -> Harness {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("../zealot-infra/migrations/sqlite")
        .run(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO account (account_id, username, email, password) VALUES (1, 'u', 'u@example.com', 'x')",
    )
    .execute(&pool)
    .await
    .unwrap();
    let events = Arc::new(RecordingEvents::default());
    let ports = ZealotPorts {
        media: Arc::new(MediaInMemoryPort::new()),
        password: Arc::new(BcryptPasswordPort::new()),
        events: events.clone(),
        rule_runner: Arc::new(NoopRuleRunner),
    };
    let repos: ZealotRepos = get_sqlite_repos(pool);
    let services = Arc::new(ZealotServices::new(ports, repos.clone()));
    let repos = Arc::new(repos);
    let runner = LuaRuleRunner::new(repos.clone(), services.clone());
    Harness {
        repos,
        services,
        runner,
        events,
    }
}

impl Harness {
    fn add_rule(&self, trigger: TriggerKind) {
        self.repos
            .rule
            .add_rule(
                &AddRuleDto {
                    name: "probe".into(),
                    description: None,
                    trigger,
                    script: "-- body is irrelevant; the dispatch result is counted".into(),
                    enabled: Some(true),
                },
                &Id::try_from(1).unwrap(),
            )
            .unwrap();
    }

    /// Run a script as account 1 (its writes emit events on the recording port).
    async fn write(&self, script: &str) {
        let lua = new_sandbox().unwrap();
        setup_zealot_globals(
            &lua,
            &RuleContext::Manual,
            self.services.clone(),
            Id::try_from(1).unwrap(),
            Arc::new(Mutex::new(Vec::new())),
        )
        .unwrap();
        execute_script(lua, script.to_string())
            .await
            .unwrap_or_else(|e| panic!("script failed: {e}\n{script}"));
    }

    /// Dispatch every event emitted since the last call; return how many rule runs each caused.
    async fn dispatch(&self) -> Vec<(&'static str, usize)> {
        let events: Vec<ZealotEvent> = std::mem::take(&mut *self.events.0.lock().unwrap());
        let mut runs = Vec::new();
        for event in events {
            let kind = event.trigger_kind();
            runs.push((kind, self.runner.run_event_rules(event).await.len()));
        }
        runs
    }

    /// Total rule runs caused by everything emitted since the last call.
    async fn runs(&self) -> usize {
        self.dispatch().await.iter().map(|(_, n)| n).sum()
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn status_rule_ignores_other_attributes_and_unchanged_status() {
    let h = harness().await;
    h.add_rule(TriggerKind::OnAttributeSet {
        attribute_key: Some("Status".into()),
    });

    h.write(
        r#"
        local item = zealot.items.create("Child")
        zealot.items.set_attribute(item.id, "Status", "Complete")
        "#,
    )
    .await;
    assert_eq!(h.runs().await, 1, "first Complete is a real Status change");

    // Z212 reproduction: a write to some other attribute of a Complete item.
    h.write(
        r#"
        zealot.items.set_attribute(zealot.items.get_by_title("Child").id, "Priority", 5)
        "#,
    )
    .await;
    assert_eq!(
        h.runs().await,
        0,
        "Priority write must not run the Status rule"
    );

    h.write(
        r#"
        zealot.items.set_attribute(zealot.items.get_by_title("Child").id, "Status", "Complete")
        "#,
    )
    .await;
    assert_eq!(
        h.runs().await,
        0,
        "Status re-set to the same value is no change"
    );

    h.write(
        r#"
        zealot.items.set_attribute(zealot.items.get_by_title("Child").id, "Status", "Working")
        "#,
    )
    .await;
    assert_eq!(h.runs().await, 1, "a changed Status runs the rule");
}

#[tokio::test(flavor = "multi_thread")]
async fn create_and_assign_type_do_not_run_an_attribute_set_rule() {
    let h = harness().await;
    h.add_rule(TriggerKind::OnAttributeSet {
        attribute_key: Some("Status".into()),
    });
    h.write(
        r#"
        local item = zealot.items.create("Typed")
        zealot.items.set_attribute(item.id, "Status", "To Do")
        zealot.items.set_attribute(item.id, "Priority", 5)
        "#,
    )
    .await;
    h.dispatch().await; // the setup writes above are real attribute changes
    h.write(r#"zealot.items.assign_type(zealot.items.get_by_title("Typed").id, "Plan")"#)
        .await;
    h.services
        .item
        .add_item(
            &AddItemDto {
                title: "Made complete".into(),
                content: String::new(),
                attributes: Some(HashMap::from([("Status".into(), json!("Complete"))])),
                types: None,
                links: None,
            },
            &make_account(Id::try_from(1).unwrap()).unwrap(),
        )
        .unwrap();
    let runs = h.dispatch().await;
    assert!(!runs.is_empty(), "create and assign_type emit events");
    assert!(runs.iter().all(|(_, n)| *n == 0), "{runs:?}");
}

#[tokio::test(flavor = "multi_thread")]
async fn unfiltered_rule_runs_on_a_change_but_not_on_an_unchanged_value() {
    let h = harness().await;
    h.add_rule(TriggerKind::OnAttributeSet {
        attribute_key: None,
    });
    h.write(
        r#"
        local item = zealot.items.create("Any")
        zealot.items.set_attribute(item.id, "Status", "Working")
        "#,
    )
    .await;
    assert_eq!(h.runs().await, 1);
    h.write(
        r#"
        zealot.items.set_attribute(zealot.items.get_by_title("Any").id, "Status", "Working")
        "#,
    )
    .await;
    assert_eq!(h.runs().await, 0);
}

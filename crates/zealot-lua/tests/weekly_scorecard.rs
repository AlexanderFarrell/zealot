//! Runs the documented weekly-scorecard example (docs/examples/weekly-scorecard.lua)
//! against an in-memory sqlite test database. Never touches a real server.

use std::sync::{Arc, Mutex};

use sqlx::sqlite::SqlitePoolOptions;
use zealot_app::{
    ports::{
        ZealotPorts,
        events::{EventPort, ZealotEvent},
        rule_runner::{NoopRuleRunner, RuleContext},
    },
    services::ZealotServices,
};
use zealot_domain::common::id::Id;
use zealot_infra::{
    ports::{media::in_memory::MediaInMemoryPort, password::bcrypt_password::BcryptPasswordPort},
    repos::sqlite::get_sqlite_repos,
};
use zealot_lua::{
    bindings::setup_zealot_globals,
    sandbox::{execute_script, new_sandbox},
};

#[derive(Debug, Default)]
struct RecordingEvents(Mutex<Vec<(String, String)>>);

impl EventPort for RecordingEvents {
    fn emit(&self, event: ZealotEvent) {
        if let ZealotEvent::AttributeSet {
            item,
            attribute_key,
            ..
        } = event
        {
            self.0
                .lock()
                .unwrap()
                .push((item.title.clone(), attribute_key));
        }
    }
}

#[allow(dead_code)]
struct Harness {
    services: Arc<ZealotServices>,
    pool: sqlx::SqlitePool,
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
    let services = Arc::new(ZealotServices::new(ports, get_sqlite_repos(pool.clone())));
    Harness {
        services,
        pool,
        events,
    }
}

#[allow(dead_code)]
impl Harness {
    /// Run a script as account 1 and return everything it passed to `zealot.notify`.
    async fn run(&self, script: &str) -> String {
        let buf = Arc::new(Mutex::new(Vec::new()));
        let lua = new_sandbox().unwrap();
        setup_zealot_globals(
            &lua,
            &RuleContext::Manual,
            self.services.clone(),
            Id::try_from(1).unwrap(),
            buf.clone(),
        )
        .unwrap();
        execute_script(lua, script.to_string())
            .await
            .unwrap_or_else(|e| panic!("script failed: {e}\n{script}"));
        let out = buf.lock().unwrap().join("\n");
        out
    }

    async fn run_err(&self, script: &str) -> String {
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
            .expect_err("script should fail")
            .to_string()
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn weekly_scorecard_example_runs() {
    use chrono::{Datelike, Duration, Utc};
    let h = harness().await;
    let today = Utc::now().date_naive();
    let monday = today - Duration::days(today.weekday().num_days_from_monday() as i64);
    let last_week = monday - Duration::days(3);
    let fmt = |d: chrono::NaiveDate| d.format("%Y-%m-%d").to_string();

    // Tickets: two shipped this week, one last week, one still open.
    h.run(
        r#"
        for _, t in ipairs({"Alpha", "Bravo", "Charlie", "Delta", "Craft sprint", "Run"}) do
            zealot.items.create(t)
        end
        "#,
    )
    .await;
    sqlx::query("INSERT INTO item_type (name, description, account_id) VALUES ('Ticket', 't', 1)")
        .execute(&h.pool)
        .await
        .unwrap();
    for title in ["Alpha", "Bravo", "Charlie", "Delta"] {
        sqlx::query("INSERT INTO item_item_type_link (item_id, type_id) SELECT i.item_id, t.type_id FROM item i, item_type t WHERE i.title = ? AND t.name = 'Ticket'")
            .bind(title).execute(&h.pool).await.unwrap();
    }
    for (title, status, done) in [
        ("Alpha", "Complete", Some(fmt(monday))),
        (
            "Bravo",
            "Complete",
            Some(format!("{} 09:00:00", fmt(today))),
        ),
        ("Charlie", "Complete", Some(fmt(last_week))),
        ("Delta", "Working", None),
    ] {
        sqlx::query("INSERT INTO attribute (item_id, key, value_text) SELECT item_id, 'Status', ? FROM item WHERE title = ?")
            .bind(status).bind(title).execute(&h.pool).await.unwrap();
        if let Some(done) = done {
            sqlx::query("INSERT INTO attribute (item_id, key, value_text) SELECT item_id, 'Completed At', ? FROM item WHERE title = ?")
                .bind(done).bind(title).execute(&h.pool).await.unwrap();
        }
    }

    // Evidence comments today, a Craft block with a receipt, and a habit.
    h.run(
        r#"
        local a = zealot.items.get_by_title("Alpha")
        zealot.comments.add(a.id, "evidence one")
        zealot.comments.add(a.id, "evidence two")
        "#,
    )
    .await;
    sqlx::query("INSERT INTO time_block (item_id, date, start_min, end_min, note, account_id) SELECT item_id, ?, 600, 750, 'receipt', 1 FROM item WHERE title = 'Craft sprint'")
        .bind(fmt(monday)).execute(&h.pool).await.unwrap();
    sqlx::query("INSERT INTO item_item_type_link (item_id, type_id) SELECT i.item_id, t.type_id FROM item i, item_type t WHERE i.title = 'Run' AND t.name = 'Repeat'")
        .execute(&h.pool).await.unwrap();
    sqlx::query("INSERT INTO attribute (item_id, key, value_text) SELECT item_id, 'Schedule', '1111111' FROM item WHERE title = 'Run'")
        .execute(&h.pool).await.unwrap();
    sqlx::query("INSERT INTO repeat_entry (item_id, date, status, comment) SELECT item_id, ?, 'Complete', '' FROM item WHERE title = 'Run'")
        .bind(fmt(monday)).execute(&h.pool).await.unwrap();

    let script = include_str!("../../../docs/examples/weekly-scorecard.lua");
    let out = h.run(script).await;
    println!("{out}");
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(
        lines[0],
        format!(
            "Scorecard {} to {} (UTC)",
            fmt(monday),
            fmt(monday + Duration::days(6))
        )
    );
    assert_eq!(lines[1], "Shipped: 2");
    assert_eq!(lines[2], "Evidence comments: 2");
    assert_eq!(lines[3], "Craft: 2.5 h (1 blocks with receipts)");
    assert!(lines[4].starts_with("Habit Run: 1 complete"), "{out}");
}

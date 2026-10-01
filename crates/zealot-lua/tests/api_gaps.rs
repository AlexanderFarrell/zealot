//! Lua API coverage for Z192: reporting reads, remove_attribute, paging, id handling,
//! and the time zone clock. Runs scripts against an in-memory sqlite database.

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
async fn comments_list_and_for_date() {
    let h = harness().await;
    let out = h
        .run(
            r#"
            local item = zealot.items.create("Commented")
            zealot.comments.add(item.id, "first")
            zealot.comments.add(item.id, "second")
            local list = zealot.comments.list(item.id)
            zealot.notify(#list .. " " .. list[1].content .. " " .. list[1].title .. " " .. list[1].item_id)
            local today = string.sub(list[1].timestamp, 1, 10)
            zealot.notify(#zealot.comments.for_date(today) .. " today")
            zealot.notify(#zealot.comments.for_date("2001-01-01") .. " other")
            "#,
        )
        .await;
    let lines: Vec<&str> = out.lines().collect();
    assert!(lines[0].starts_with("2 first Commented"), "{out}");
    assert_eq!(lines[1], "2 today");
    assert_eq!(lines[2], "0 other");
}

#[tokio::test(flavor = "multi_thread")]
async fn habits_entries_and_stats() {
    let h = harness().await;
    h.run(r#"zealot.items.create("Run")"#).await;
    sqlx::query(
        "INSERT INTO item_item_type_link (item_id, type_id) SELECT i.item_id, t.type_id FROM item i, item_type t WHERE i.title = 'Run' AND t.name = 'Repeat'",
    )
    .execute(&h.pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO attribute (item_id, key, value_text) SELECT item_id, 'Schedule', '1111111' FROM item WHERE title = 'Run'",
    )
    .execute(&h.pool)
    .await
    .unwrap();
    for (date, status) in [
        ("2020-01-01", "Complete"),
        ("2020-01-02", "Complete"),
        ("2020-01-03", "Skip"),
        ("2020-01-04", "Complete"),
    ] {
        sqlx::query(
            "INSERT INTO repeat_entry (item_id, date, status, comment) SELECT item_id, ?, ?, 'c' FROM item WHERE title = 'Run'",
        )
        .bind(date)
        .bind(status)
        .execute(&h.pool)
        .await
        .unwrap();
    }
    let out = h
        .run(
            r#"
            zealot.notify(#zealot.habits.list() .. " habits")
            local e = zealot.habits.entries("2020-01-01", "2020-01-04")
            zealot.notify(#e .. " entries " .. e[1].title .. " " .. e[1].status .. " " .. e[1].comment)
            zealot.notify(#zealot.habits.entries("2020-01-02") .. " single")
            local s = zealot.habits.stats("2020-01-01", "2020-01-04")
            local r = s.habits[1]
            zealot.notify(s.habit_count .. " " .. r.entries .. " " .. r.counts.complete .. " " ..
                r.counts.skip .. " " .. r.current_streak .. " " .. r.longest_streak .. " " .. r.completion_rate)
            "#,
        )
        .await;
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "1 habits");
    assert_eq!(lines[1], "4 entries Run Complete c");
    assert_eq!(lines[2], "1 single");
    assert_eq!(lines[3], "1 4 3 1 3 3 1.0");

    let err = h
        .run_err(r#"zealot.habits.stats("2019-01-01", "2020-12-31")"#)
        .await;
    assert!(err.contains("capped at 366"), "{err}");
}

#[tokio::test(flavor = "multi_thread")]
async fn time_blocks_by_range_and_item() {
    let h = harness().await;
    let item_id = h
        .run(r#"zealot.notify(zealot.items.create("Craft").id)"#)
        .await;
    for (date, start, end) in [
        ("2026-09-28", 600, 690),
        ("2026-09-29", 540, 600),
        ("2026-10-10", 600, 660),
    ] {
        sqlx::query(
            "INSERT INTO time_block (item_id, date, start_min, end_min, note, account_id) VALUES (?, ?, ?, ?, 'n', 1)",
        )
        .bind(item_id.parse::<i64>().unwrap())
        .bind(date)
        .bind(start)
        .bind(end)
        .execute(&h.pool)
        .await
        .unwrap();
    }
    let out = h
        .run(&format!(
            r#"
            local week = zealot.time_blocks.for_range("2026-09-28", "2026-10-04")
            local minutes = 0
            for _, b in ipairs(week) do minutes = minutes + b.minutes end
            zealot.notify(#week .. " " .. minutes .. " " .. week[1].start .. "-" .. week[1]["end"] .. " " .. week[1].title)
            zealot.notify(#zealot.time_blocks.for_item("{item_id}") .. " by item")
            zealot.notify(#zealot.time_blocks.for_range("2026-09-28") .. " one day")
            "#
        ))
        .await;
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "2 150 10:00-11:30 Craft");
    assert_eq!(lines[1], "3 by item");
    assert_eq!(lines[2], "1 one day");
}

#[tokio::test(flavor = "multi_thread")]
async fn remove_attribute_clears_value_and_fires_event() {
    let h = harness().await;
    let out = h
        .run(
            r#"
            local item = zealot.items.create("Reopen me")
            zealot.items.set_attribute(item.id, "Note", "x")
            zealot.notify(tostring(zealot.items.get(item.id).attributes["Note"]))
            zealot.items.remove_attribute(item.id, "Note")
            zealot.notify(tostring(zealot.items.get(item.id).attributes["Note"]))
            "#,
        )
        .await;
    let events = h.events.0.lock().unwrap().clone();
    assert_eq!(out.lines().last().unwrap(), "nil", "{out}");
    let keys: Vec<_> = events.iter().map(|(_, k)| k.as_str()).collect();
    assert_eq!(
        keys,
        vec!["Note", "Note"],
        "set then remove both fire AttributeSet"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn search_pages_and_never_truncates() {
    let h = harness().await;
    let out = h
        .run(
            r#"
            for i = 1, 45 do zealot.items.create("Needle " .. i) end
            zealot.notify(#zealot.items.search("Needle") .. " all")
            zealot.notify(#zealot.items.search("Needle", {limit = 10}) .. " limit")
            zealot.notify(#zealot.items.search("Needle", {limit = 10, offset = 40}) .. " tail")
            "#,
        )
        .await;
    assert_eq!(out, "45 all\n10 limit\n5 tail");
    let err = h
        .run_err(r#"zealot.items.search("Needle", {limit = 0})"#)
        .await;
    assert!(err.contains("limit must be at least 1"), "{err}");
}

#[tokio::test(flavor = "multi_thread")]
async fn ids_accept_numbers_and_numeric_strings() {
    let h = harness().await;
    let ids = h
        .run(
            r#"
            local parent = zealot.items.create("Parent")
            local child = zealot.items.create("Child")
            zealot.notify(parent.id)
            zealot.notify(child.id)
            "#,
        )
        .await;
    let (parent, child) = ids.split_once('\n').unwrap();
    sqlx::query("INSERT INTO attribute_list_value (item_id, key, ordinal, value_item_id) VALUES (?, 'Parent', 0, ?)")
        .bind(child.parse::<i64>().unwrap())
        .bind(parent.parse::<i64>().unwrap())
        .execute(&h.pool)
        .await
        .unwrap();
    let out = h
        .run(&format!(
            r#"
            local sid = "{parent}"
            zealot.notify(zealot.items.get(sid).title)
            zealot.notify(#zealot.items.filter({{{{key = "Parent", op = "eq", value = {parent}}}}}) .. " number")
            zealot.notify(#zealot.items.filter({{{{key = "Parent", op = "eq", value = sid}}}}) .. " string")
            zealot.items.set_attribute(sid, "Note", "x")
            zealot.items.remove_attribute(sid, "Note")
            zealot.items.update(sid, {{content = "hi"}})
            "#
        ))
        .await;
    assert_eq!(out, "Parent\n1 number\n1 string");
    let err = h.run_err(r#"zealot.items.get("abc")"#).await;
    assert!(err.contains("i64") || err.contains("integer"), "{err}");
}

#[tokio::test(flavor = "multi_thread")]
async fn empty_filter_and_list_enumerate_with_paging() {
    let h = harness().await;
    let out = h
        .run(
            r#"
            for i = 1, 7 do zealot.items.create("Thing " .. i) end
            local all = zealot.items.filter({})
            zealot.notify(#all .. " all")
            zealot.notify(#zealot.items.list() .. " list")
            zealot.notify(#zealot.items.filter({}, {limit = 3}) .. " page1")
            zealot.notify(#zealot.items.list({limit = 3, offset = 6}) .. " tail")
            "#,
        )
        .await;
    assert_eq!(out, "7 all\n7 list\n3 page1\n1 tail");
}

#[tokio::test(flavor = "multi_thread")]
async fn clock_uses_account_timezone_then_utc() {
    let h = harness().await;
    let out = h
        .run(
            r#"
            zealot.notify(zealot.timezone)
            zealot.notify(tostring(#zealot.date == 10))
            zealot.notify(zealot.local_date("2026-07-01 23:30:00"))
            zealot.notify(zealot.date_add("2026-10-01", 7))
            zealot.notify(zealot.week_start("2026-10-01"))
            "#,
        )
        .await;
    assert_eq!(out, "UTC\ntrue\n2026-07-01\n2026-10-08\n2026-09-28");

    sqlx::query(
        "UPDATE account SET settings = '{\"timezone\":\"Europe/London\"}' WHERE account_id = 1",
    )
    .execute(&h.pool)
    .await
    .unwrap();
    let out = h
        .run(
            r#"
            zealot.notify(zealot.timezone)
            zealot.notify(zealot.local_date("2026-07-01 23:30:00"))
            zealot.notify(zealot.local_date("2026-01-01 23:30:00"))
            "#,
        )
        .await;
    assert_eq!(out, "Europe/London\n2026-07-02\n2026-01-01");
}

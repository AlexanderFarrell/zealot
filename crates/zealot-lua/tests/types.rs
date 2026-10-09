//! Lua type and attribute kind reflection (Z219): `zealot.types` and `zealot.attribute_kinds`.
//! Runs scripts against an in-memory sqlite database.

use std::sync::{Arc, Mutex};

use serde_json::json;
use sqlx::sqlite::SqlitePoolOptions;
use zealot_app::{
    ports::{
        ZealotPorts,
        events::NoopEventPort,
        rule_runner::{NoopRuleRunner, RuleContext},
    },
    services::ZealotServices,
};
use zealot_domain::{attribute::AddAttributeKindDto, common::id::Id, item_type::AddItemTypeDto};
use zealot_infra::{
    ports::{media::in_memory::MediaInMemoryPort, password::bcrypt_password::BcryptPasswordPort},
    repos::sqlite::get_sqlite_repos,
};
use zealot_lua::{
    bindings::setup_zealot_globals,
    sandbox::{execute_script, new_sandbox},
};

fn me() -> Id {
    Id::try_from(1).unwrap()
}

async fn services() -> Arc<ZealotServices> {
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
    let ports = ZealotPorts {
        media: Arc::new(MediaInMemoryPort::new()),
        password: Arc::new(BcryptPasswordPort::new()),
        events: Arc::new(NoopEventPort),
        rule_runner: Arc::new(NoopRuleRunner),
    };
    Arc::new(ZealotServices::new(ports, get_sqlite_repos(pool)))
}

async fn run(svc: &Arc<ZealotServices>, script: &str) -> String {
    let buf = Arc::new(Mutex::new(Vec::new()));
    let lua = new_sandbox().unwrap();
    setup_zealot_globals(&lua, &RuleContext::Manual, svc.clone(), me(), buf.clone()).unwrap();
    execute_script(lua, script.to_string())
        .await
        .unwrap_or_else(|e| panic!("script failed: {e}\n{script}"));
    let out = buf.lock().unwrap().join("\n");
    out
}

/// A user type requiring Status (system, has a default) and a user kind with a default.
async fn seeded() -> Arc<ZealotServices> {
    let svc = services().await;
    svc.attribute
        .add_attribute_kind(
            &AddAttributeKindDto {
                key: "Effort".into(),
                description: "Effort points".into(),
                base_type: "integer".into(),
                config: json!({"min": 1, "max": 10, "default": 3}),
            },
            &me(),
        )
        .unwrap();
    svc.item_type
        .add_item_type(
            &AddItemTypeDto {
                name: "Scrap".into(),
                description: String::new(),
                required_attributes: vec![],
            },
            &me(),
        )
        .unwrap();
    svc.item_type
        .add_item_type(
            &AddItemTypeDto {
                name: "Chore".into(),
                description: "A small job".into(),
                required_attributes: vec!["Status".into(), "Effort".into()],
            },
            &me(),
        )
        .unwrap();
    svc
}

#[tokio::test(flavor = "multi_thread")]
async fn types_get_returns_definition_with_required_attribute_detail() {
    let svc = seeded().await;
    let out = run(
        &svc,
        r#"
        local t = zealot.types.get("Chore")
        zealot.notify(t.name .. "|" .. t.description .. "|" .. tostring(t.is_system) .. "|" .. t.item_count)
        zealot.notify(#t.required_attributes)
        local status, effort = t.required_attributes[1], t.required_attributes[2]
        zealot.notify(status.key .. "|" .. status.base_type .. "|" .. tostring(status.is_system) .. "|" .. tostring(status.default))
        zealot.notify(effort.key .. "|" .. effort.base_type .. "|" .. effort.config.min .. "|" .. effort.config.max .. "|" .. effort.default)
        "#,
    )
    .await;
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "Chore|A small job|false|0", "{out}");
    assert_eq!(lines[1], "2");
    assert_eq!(lines[2], "Status|dropdown|true|To Do");
    assert_eq!(lines[3], "Effort|integer|1|10|3");
}

#[tokio::test(flavor = "multi_thread")]
async fn types_get_unknown_is_nil_and_item_count_tracks_items() {
    let svc = seeded().await;
    let out = run(
        &svc,
        r#"
        zealot.notify(tostring(zealot.types.get("Nope")))
        zealot.items.create("Sweep", "", { types = { "Scrap" } })
        zealot.notify(zealot.types.get("Scrap").item_count)
        "#,
    )
    .await;
    assert_eq!(out, "nil\n1");
}

#[tokio::test(flavor = "multi_thread")]
async fn types_list_is_sorted_and_includes_system_and_user_types() {
    let svc = seeded().await;
    let out = run(
        &svc,
        r#"
        local names, prev, sorted, system, user = {}, "", true, false, false
        for _, t in ipairs(zealot.types.list()) do
            if t.name < prev then sorted = false end
            prev = t.name
            if t.is_system then system = true else user = true end
            names[#names + 1] = t.name
        end
        zealot.notify(tostring(sorted) .. " " .. tostring(system) .. " " .. tostring(user))
        zealot.notify(table.concat(names, ",") .. ",")
        "#,
    )
    .await;
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "true true true", "{out}");
    assert!(lines[1].contains("Chore,"), "{out}");
}

#[tokio::test(flavor = "multi_thread")]
async fn attribute_kinds_get_and_list() {
    let svc = seeded().await;
    let out = run(
        &svc,
        r#"
        local k = zealot.attribute_kinds.get("Effort")
        zealot.notify(k.key .. "|" .. k.description .. "|" .. k.base_type .. "|" .. tostring(k.is_system) .. "|" .. k.config.default .. "|" .. k.default)
        local s = zealot.attribute_kinds.get("Status")
        zealot.notify(s.base_type .. "|" .. tostring(s.is_system) .. "|" .. #s.config.values .. "|" .. s.default)
        zealot.notify(tostring(zealot.attribute_kinds.get("Nope")))
        local found = false
        for _, kind in ipairs(zealot.attribute_kinds.list()) do
            if kind.key == "Effort" then found = true end
        end
        zealot.notify(tostring(found))
        "#,
    )
    .await;
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "Effort|Effort points|integer|false|3|3", "{out}");
    assert!(lines[1].starts_with("dropdown|true|"), "{out}");
    assert!(lines[1].ends_with("|To Do"), "{out}");
    assert_eq!(lines[2], "nil");
    assert_eq!(lines[3], "true");
}

#[tokio::test(flavor = "multi_thread")]
async fn schema_is_read_only_from_lua() {
    let svc = seeded().await;
    let out = run(
        &svc,
        r#"zealot.notify(tostring(zealot.types.create) .. tostring(zealot.types.update) .. tostring(zealot.attribute_kinds.create))"#,
    )
    .await;
    assert_eq!(out, "nilnilnil");
}

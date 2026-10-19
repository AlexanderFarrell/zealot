//! The query extensions saved views rely on (`exists`/`missing`, `created`/`updated`, relative
//! date tokens, `any_of`, `linked_from`), run through `ItemService::query_items` against real
//! SQLite and PostgreSQL. The seed models the October 2026 planning views. No production data.
use super::{postgres::get_postgres_repos, sqlite::get_sqlite_repos};
use chrono::{Datelike, Utc};
use serde_json::{Value, json};
use sqlx::{SqlitePool, postgres::PgPoolOptions, sqlite::SqlitePoolOptions};
use std::sync::Arc;
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{ImageExt, runners::AsyncRunner},
};
use zealot_app::{
    ports::events::{EventPort, NoopEventPort},
    repos::ZealotRepos,
    services::item::ItemService,
};
use zealot_domain::{
    account::Account,
    attribute::AttributeFilterDto,
    common::{email::Email, id::Id},
    item::ItemQueryDto,
};

/// Items: 1 the monthly plan (wikilinks 2 and 6); Tickets 2 (Date today), 3 (Week this week),
/// 4 (Working, no Date or Week, stale, LLM), 5 (To Do, LLM); 6 a Working Book.
const SEED: &[&str] = &[
    "INSERT INTO account (account_id, username, email, password) VALUES (1, 'u', 'u@example.com', 'x')",
    "INSERT INTO item_type (name, description, account_id) VALUES ('Ticket', 't', 1)",
    "INSERT INTO item (item_id, title, content, account_id) VALUES
        (1, 'October 2026', '[[Ticket A]] [[Some Book]]', 1), (2, 'Ticket A', '', 1),
        (3, 'Ticket B', '', 1), (4, 'Ticket C', '', 1), (5, 'Ticket D', '', 1), (6, 'Some Book', '', 1)",
    "INSERT INTO item_item_type_link (item_id, type_id)
        SELECT i.item_id, t.type_id FROM item i, item_type t
        WHERE i.item_id IN (2, 3, 4, 5) AND t.name = 'Ticket' AND t.account_id = 1",
    "INSERT INTO item_item_link (first_item_id, second_item_id, relationship) VALUES
        (1, 2, 'wikilink'), (1, 6, 'wikilink'), (2, 1, 'parent')",
    "INSERT INTO attribute (item_id, key, value_text) VALUES
        (2, 'Status', 'Working'), (3, 'Status', 'Working'), (4, 'Status', 'Working'),
        (5, 'Status', 'To Do'), (6, 'Status', 'Working'),
        (4, 'Executor', 'LLM'), (5, 'Executor', 'LLM'), (2, 'Executor', 'Alexander')",
];

fn id(n: i64) -> Id {
    Id::try_from(n).unwrap()
}

fn account() -> Account {
    Account {
        account_id: id(1),
        username: "u".into(),
        email: Email::try_from("u@example.com".to_string()).unwrap(),
        given_name: "".into(),
        surname: "".into(),
        // Pin the clock so tokens resolve the same way the seed computed "today".
        settings: json!({"timezone": "UTC"}),
        has_api_key: false,
    }
}

fn service(repos: &ZealotRepos) -> ItemService {
    let events: Arc<dyn EventPort> = Arc::new(NoopEventPort);
    ItemService::new(
        &repos.item,
        &repos.item_attribute_value,
        &repos.item_external_link,
        &repos.item_heading,
        &repos.item_link,
        &repos.item_type,
        &repos.attribute,
        &events,
    )
}

fn today() -> String {
    Utc::now().date_naive().format("%Y-%m-%d").to_string()
}

fn this_week() -> String {
    let week = Utc::now().date_naive().iso_week();
    format!("{}-W{:02}", week.year(), week.week())
}

fn filter(key: &str, op: &str, value: Value) -> AttributeFilterDto {
    AttributeFilterDto {
        key: key.into(),
        op: op.into(),
        value,
        list_mode: "any".into(),
    }
}

fn ids(svc: &ItemService, q: &ItemQueryDto) -> Vec<i64> {
    let (items, total) = svc.query_items(q, &account()).unwrap();
    let ids: Vec<i64> = items.iter().map(|i| i64::from(i.item_id)).collect();
    assert_eq!(ids.len() as i64, total);
    ids
}

fn verify(repos: ZealotRepos) {
    let svc = service(&repos);
    let tickets = || ItemQueryDto {
        types: vec!["Ticket".into()],
        ..Default::default()
    };

    // Today: Date is today OR Week is this week, written with tokens.
    let q = ItemQueryDto {
        any_of: vec![
            vec![filter("Date", "eq", json!("$today"))],
            vec![filter("Week", "eq", json!("$this_week"))],
        ],
        ..Default::default()
    };
    assert_eq!(ids(&svc, &q), [2, 3]);

    // any_of is ANDed with the plain filters.
    let mut q2 = q.clone();
    q2.filters = vec![filter("Executor", "eq", json!("Alexander"))];
    assert_eq!(ids(&svc, &q2), [2]);

    // AI Queue: Executor LLM, To Do or Working.
    let q = ItemQueryDto {
        filters: vec![
            filter("Executor", "eq", json!("LLM")),
            filter("Status", "eq", json!(["To Do", "Working"])),
        ],
        ..Default::default()
    };
    assert_eq!(ids(&svc, &q), [4, 5]);

    // exists / missing need no value.
    let q = ItemQueryDto {
        filters: vec![filter("Executor", "exists", Value::Null)],
        ..Default::default()
    };
    assert_eq!(ids(&svc, &q), [2, 4, 5]);

    // Operating Health: Working Tickets with no Date and no Week.
    let mut q = tickets();
    q.filters = vec![
        filter("Status", "eq", json!("Working")),
        filter("Date", "missing", Value::Null),
        filter("Week", "missing", Value::Null),
    ];
    assert_eq!(ids(&svc, &q), [4]);

    // Operating Health: Working, not updated in 30 days.
    let q = ItemQueryDto {
        filters: vec![
            filter("Status", "eq", json!("Working")),
            filter("updated", "lt", json!("$today-30d")),
        ],
        ..Default::default()
    };
    assert_eq!(ids(&svc, &q), [4]);

    // created/updated compare whole UTC days.
    let mut q = tickets();
    q.filters = vec![filter("created", "eq", json!("$today"))];
    assert_eq!(ids(&svc, &q), [2, 3, 4, 5]);
    q.filters = vec![
        filter("updated", "gte", json!("2026-01-01")),
        filter("updated", "lte", json!("2026-01-01")),
    ];
    assert_eq!(ids(&svc, &q), [4]);

    // Month cascade: what the monthly plan links to.
    let q = ItemQueryDto {
        linked_from: Some(1),
        ..Default::default()
    };
    assert_eq!(ids(&svc, &q), [2, 6]);
    let mut q = tickets();
    q.linked_from = Some(1);
    assert_eq!(ids(&svc, &q), [2]);

    // Rejected inputs.
    let bad = |q: ItemQueryDto| assert!(svc.query_items(&q, &account()).is_err(), "{q:?}");
    let mut q = tickets();
    q.filters = vec![filter("created", "ilike", json!("2026"))];
    bad(q.clone());
    q.filters = vec![filter("created", "eq", json!("soon"))];
    bad(q.clone());
    q.filters = vec![filter("Date", "eq", json!("$today+3"))];
    bad(q.clone());
    q.filters = vec![];
    q.any_of = vec![vec![]];
    bad(q);
    bad(ItemQueryDto::default());
}

async fn sqlite() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("migrations/sqlite")
        .run(&pool)
        .await
        .unwrap();
    for statement in SEED {
        sqlx::query(statement).execute(&pool).await.unwrap();
    }
    let backend = [
        format!(
            "INSERT INTO attribute (item_id, key, value_date) VALUES (2, 'Date', strftime('%s', '{}'))",
            today()
        ),
        format!(
            "INSERT INTO attribute (item_id, key, value_text) VALUES (3, 'Week', '{}')",
            this_week()
        ),
        String::from(
            "UPDATE item SET updated_at = strftime('%s', '2026-01-01 12:00:00') WHERE item_id = 4",
        ),
    ];
    for statement in backend {
        sqlx::query(&statement).execute(&pool).await.unwrap();
    }
    pool
}

#[tokio::test(flavor = "multi_thread")]
async fn view_queries_on_sqlite() {
    verify(get_sqlite_repos(sqlite().await));
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires Docker"]
async fn view_queries_on_postgres() {
    let container = Postgres::default()
        .with_tag("16-alpine")
        .start()
        .await
        .unwrap();
    let port = container.get_host_port_ipv4(5432).await.unwrap();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&format!(
            "postgres://postgres:postgres@127.0.0.1:{port}/postgres"
        ))
        .await
        .unwrap();
    sqlx::migrate!("migrations/postgres")
        .run(&pool)
        .await
        .unwrap();
    for statement in SEED {
        sqlx::query(statement).execute(&pool).await.unwrap();
    }
    let backend = [
        format!(
            "INSERT INTO attribute (item_id, key, value_date) VALUES (2, 'Date', '{}')",
            today()
        ),
        format!(
            "INSERT INTO attribute (item_id, key, value_text) VALUES (3, 'Week', '{}')",
            this_week()
        ),
        String::from("UPDATE item SET updated_at = '2026-01-01T12:00:00Z' WHERE item_id = 4"),
    ];
    for statement in backend {
        sqlx::query(&statement).execute(&pool).await.unwrap();
    }
    verify(get_postgres_repos(pool));
}

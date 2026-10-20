//! Saved Views end to end through `ViewService` and real SQLite and PostgreSQL: the
//! `Saved View` type from migration 0021, definitions read from content, paging, grouping,
//! inspect and validate. No production data.
use super::{postgres::get_postgres_repos, sqlite::get_sqlite_repos};
use serde_json::json;
use sqlx::{SqlitePool, postgres::PgPoolOptions, sqlite::SqlitePoolOptions};
use std::sync::Arc;
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{ImageExt, runners::AsyncRunner},
};
use zealot_app::{
    ports::events::{EventPort, NoopEventPort},
    repos::ZealotRepos,
    services::{
        item::ItemService,
        view::{ViewService, ViewServiceError},
    },
};
use zealot_domain::{
    account::Account,
    common::{email::Email, id::Id},
    saved_view::ViewDisplay,
};

/// Projects 1 and 2; LLM Plans 3, 4 (under 1) and 5 (under 2); 6 an Alexander Plan.
/// View items: 10 AI Queue (grouped by Parent, limit 2), 11 untyped, 12 broken JSON,
/// 13 a typo'd query key, 14 a token-based Today view.
const SEED: &[&str] = &[
    "INSERT INTO account (account_id, username, email, password) VALUES (1, 'u', 'u@example.com', 'x'), (2, 'v', 'v@example.com', 'x')",
    "INSERT INTO item (item_id, title, content, account_id) VALUES
        (1, 'Zealot', '', 1), (2, 'Elephant', '', 1),
        (3, 'Plan A', '', 1), (4, 'Plan B', '', 1), (5, 'Plan C', '', 1), (6, 'Plan D', '', 1),
        (7, 'Other account LLM plan', '', 2),
        (10, 'AI Queue', '# AI Queue

Work an agent can pick up.

```zealot-view
{\"query\": {\"filters\": [{\"key\": \"Executor\", \"op\": \"eq\", \"value\": \"LLM\"}], \"sort\": {\"key\": \"title\"}, \"limit\": 2},
 \"group_by\": \"Parent\", \"columns\": [\"Status\"]}
```
', 1),
        (11, 'Not a view', '```zealot-view
{\"query\": {\"types\": [\"Plan\"]}}
```', 1),
        (12, 'Broken view', '```zealot-view
{\"query\":
```', 1),
        (13, 'Typo view', '```zealot-view
{\"query\": {\"filter\": [], \"types\": [\"Plan\"]}}
```', 1),
        (14, 'Today', '```zealot-view
{\"query\": {\"any_of\": [[{\"key\": \"updated\", \"op\": \"gte\", \"value\": \"$today-1d\"}]], \"types\": [\"Plan\"]}}
```', 1)",
    "INSERT INTO item_item_type_link (item_id, type_id)
        SELECT i.item_id, t.type_id FROM item i, item_type t
        WHERE (i.item_id IN (10, 12, 13, 14) AND t.name = 'Saved View' AND t.account_id IS NULL)
           OR (i.item_id IN (3, 4, 5, 6, 7) AND t.name = 'Plan' AND t.account_id IS NULL)",
    "INSERT INTO attribute (item_id, key, value_text) VALUES
        (3, 'Executor', 'LLM'), (4, 'Executor', 'LLM'), (5, 'Executor', 'LLM'), (6, 'Executor', 'Alexander'),
        (7, 'Executor', 'LLM'), (3, 'Status', 'To Do'), (4, 'Status', 'Working')",
    "INSERT INTO attribute_list_value (item_id, key, ordinal, value_item_id) VALUES
        (3, 'Parent', 0, 1), (4, 'Parent', 0, 1), (5, 'Parent', 0, 2)",
    "INSERT INTO item_item_link (first_item_id, second_item_id, relationship) VALUES
        (3, 1, 'parent'), (4, 1, 'parent'), (5, 2, 'parent')",
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
        settings: json!({"timezone": "Europe/London"}),
        has_api_key: false,
    }
}

fn views(repos: &ZealotRepos) -> ViewService {
    let events: Arc<dyn EventPort> = Arc::new(NoopEventPort);
    let item = Arc::new(ItemService::new(
        &repos.item,
        &repos.item_attribute_value,
        &repos.item_external_link,
        &repos.item_heading,
        &repos.item_link,
        &repos.item_type,
        &repos.attribute,
        &events,
    ));
    ViewService::new(&item)
}

fn invalid(result: Result<impl std::fmt::Debug, ViewServiceError>, needle: &str) {
    match result {
        Err(ViewServiceError::Invalid(msg)) => assert!(msg.contains(needle), "{msg}"),
        other => panic!("expected Invalid containing '{needle}', got {other:?}"),
    }
}

fn verify(repos: ZealotRepos) {
    let svc = views(&repos);
    let user = account();

    // The definition's limit pages the run; the other account's LLM plan never appears.
    let page = svc.run(&id(10), None, None, &user).unwrap();
    assert_eq!(page.title, "AI Queue");
    assert_eq!(page.view_id, Some(10));
    assert_eq!(page.display, None);
    assert_eq!(page.total, 3);
    let ids: Vec<i64> = page.items.iter().map(|i| i.item_id).collect();
    assert_eq!(ids, [3, 4]);
    assert_eq!(page.columns, ["Status"]);

    // Grouped by Parent, labelled with the parent's title.
    assert_eq!(page.group_by.as_deref(), Some("Parent"));
    assert_eq!(page.groups.len(), 1);
    assert_eq!(page.groups[0].label, "Zealot");
    assert_eq!(page.groups[0].item_ids, [3, 4]);

    // Caller paging overrides the definition; the next page groups under Elephant.
    let page = svc.run(&id(10), Some(50), Some(2), &user).unwrap();
    assert_eq!(
        page.items.iter().map(|i| i.item_id).collect::<Vec<_>>(),
        [5]
    );
    assert_eq!(page.groups[0].label, "Elephant");
    assert_eq!(page.offset, 2);

    // Inspect reports the resolved query and the account's time zone.
    let report = svc.inspect(&id(14), &user).unwrap();
    assert_eq!(report.time_zone, "Europe/London");
    let resolved = &report.resolved_query.any_of[0][0].value;
    assert!(resolved.as_str().unwrap().starts_with("20"), "{resolved}");
    assert_eq!(report.total, 4); // every Plan in account 1 was just created
    assert_eq!(report.view_id, Some(14));

    // Typos in the query are reported, not silently ignored.
    let report = svc.inspect(&id(13), &user).unwrap();
    assert_eq!(report.warnings.len(), 1);

    // Validate checks an unsaved definition, including its query.
    let report = svc
        .validate(&json!({"query": {"types": ["Plan"], "filters": [{"key": "Executor", "op": "missing"}]}}), &user)
        .unwrap();
    assert_eq!(report.total, 0);
    assert!(report.view_id.is_none());
    invalid(
        svc.validate(&json!({"query": {}}), &user),
        "at least one of",
    );
    invalid(
        svc.validate(
            &json!({"query": {"types": ["Plan"]}, "grouping": "x"}),
            &user,
        ),
        "unknown field",
    );
    invalid(
        svc.validate(
            &json!({"query": {"filters": [{"key": "Date", "op": "eq", "value": "$today+1"}]}}),
            &user,
        ),
        "invalid date token",
    );

    // An unsaved definition runs like a saved one, with paging, grouping and display echoed.
    let preview = svc
        .run_definition(
            &json!({"query": {"types": ["Plan"], "filters": [{"key": "Executor", "op": "eq", "value": "LLM"}], "sort": {"key": "title"}},
                    "group_by": "Parent", "display": "table"}),
            Some(2),
            Some(1),
            &user,
        )
        .unwrap();
    assert!(preview.view_id.is_none());
    assert_eq!(preview.total, 3);
    assert_eq!(
        preview.items.iter().map(|i| i.item_id).collect::<Vec<_>>(),
        [4, 5]
    );
    assert_eq!(
        preview
            .groups
            .iter()
            .map(|g| g.label.as_str())
            .collect::<Vec<_>>(),
        ["Zealot", "Elephant"]
    );
    assert_eq!(preview.display, Some(ViewDisplay::Table));
    invalid(
        svc.run_definition(&json!({"query": {}, "display": "pie"}), None, None, &user),
        "unknown variant",
    );

    // Not a Saved View, a broken definition, a missing item, another account's item.
    invalid(svc.run(&id(11), None, None, &user), "not a Saved View");
    invalid(
        svc.run(&id(12), None, None, &user),
        "invalid view definition",
    );
    assert!(matches!(
        svc.run(&id(99), None, None, &user),
        Err(ViewServiceError::NotFound)
    ));
    assert!(matches!(
        svc.run(&id(7), None, None, &user),
        Err(ViewServiceError::NotFound)
    ));
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
    pool
}

#[tokio::test(flavor = "multi_thread")]
async fn saved_views_on_sqlite() {
    verify(get_sqlite_repos(sqlite().await));
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires Docker"]
async fn saved_views_on_postgres() {
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
    verify(get_postgres_repos(pool));
}

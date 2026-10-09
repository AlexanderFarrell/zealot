//! System attribute kinds are read-only through `AttributeService`, and user kinds update and delete,
//! checked against the real repositories and migrations. No production data.
//!
//! The PostgreSQL tests start a throwaway container with testcontainers, so they are `#[ignore]`d and
//! need a running Docker daemon: `cargo test -p zealot-infra system_attribute_kind -- --ignored`.
use serde_json::{Value, json};
use sqlx::{PgPool, SqlitePool, postgres::PgPoolOptions, sqlite::SqlitePoolOptions};
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{ContainerAsync, ImageExt, runners::AsyncRunner},
};
use zealot_app::services::attribute::{AttributeService, AttributeServiceError};
use zealot_domain::{
    attribute::{AddAttributeKindDto, UpdateAttributeKindDto},
    common::id::Id,
};

use super::{ZealotRepos, postgres::get_postgres_repos, sqlite::get_sqlite_repos};

/// One account and two items, the second a child of the first. `Parent` is a list kind, so its
/// value lives in `attribute_list_value`; `Status` is a scalar in `attribute`.
const SEED: &[&str] = &[
    "INSERT INTO account (account_id, username, email, password) VALUES (1, 'u', 'u@example.com', 'x')",
    "INSERT INTO item (item_id, title, content, account_id) VALUES (1, 'Root', '', 1), (2, 'Child', '', 1)",
    "INSERT INTO attribute_list_value (item_id, key, ordinal, value_item_id) VALUES (2, 'Parent', 0, 1)",
    "INSERT INTO attribute (item_id, key, value_text) VALUES (1, 'Status', 'Working')",
];

/// Raw SQL access to the test database, so one check runs on both backends.
trait RawDb {
    async fn exec(&self, sql: &str);
    async fn scalar(&self, sql: &str) -> i64;
}

impl RawDb for SqlitePool {
    async fn exec(&self, sql: &str) {
        sqlx::query(sql).execute(self).await.unwrap();
    }
    async fn scalar(&self, sql: &str) -> i64 {
        sqlx::query_scalar(sql).fetch_one(self).await.unwrap()
    }
}

impl RawDb for PgPool {
    async fn exec(&self, sql: &str) {
        sqlx::query(sql).execute(self).await.unwrap();
    }
    async fn scalar(&self, sql: &str) -> i64 {
        sqlx::query_scalar(sql).fetch_one(self).await.unwrap()
    }
}

fn account_id() -> Id {
    Id::try_from(1i64).unwrap()
}

/// Scalar and list value rows stored for `key` (a fixed test key, never user input).
async fn count_values(db: &impl RawDb, key: &str) -> i64 {
    db.scalar(&format!(
        "SELECT (SELECT count(*) FROM attribute WHERE key = '{key}') \
         + (SELECT count(*) FROM attribute_list_value WHERE key = '{key}')"
    ))
    .await
}

fn add(key: &str, base_type: &str, config: Value) -> AddAttributeKindDto {
    AddAttributeKindDto {
        key: key.into(),
        description: format!("{key} test kind"),
        base_type: base_type.into(),
        config,
    }
}

async fn check_system_kinds_reject_delete_and_update(db: &impl RawDb, repos: ZealotRepos) {
    let svc = AttributeService::new(&repos.attribute);

    for force in [false, true] {
        let result = svc.delete_attribute_kind("Parent", &account_id(), force);
        assert!(
            matches!(&result, Err(AttributeServiceError::ReadOnly(msg)) if msg.contains("Parent")),
            "force={force}: {result:?}"
        );
    }
    assert_eq!(
        count_values(db, "Parent").await,
        1,
        "a rejected forced delete keeps Parent values"
    );

    let status = svc
        .get_kind_by_key("Status", &account_id())
        .unwrap()
        .unwrap();
    assert!(status.is_system);
    let rename = UpdateAttributeKindDto {
        kind_id: i64::from(status.kind_id),
        key: Some("State".into()),
        description: None,
        base_type: Some("text".into()),
        config: None,
    };
    let result = svc.update_attribute_kind(&rename, &account_id());
    assert!(
        matches!(result, Err(AttributeServiceError::ReadOnly(_))),
        "{result:?}"
    );
    assert!(
        svc.get_kind_by_key("Status", &account_id())
            .unwrap()
            .is_some()
    );
    assert_eq!(count_values(db, "Status").await, 1);
}

async fn check_user_kinds_update_and_delete(db: &impl RawDb, repos: ZealotRepos) {
    let svc = AttributeService::new(&repos.attribute);

    let mood = svc
        .add_attribute_kind(&add("Mood", "text", json!({})), &account_id())
        .unwrap()
        .unwrap();
    let describe = UpdateAttributeKindDto {
        kind_id: i64::from(mood.kind_id),
        key: None,
        description: Some("How it felt".into()),
        base_type: None,
        config: None,
    };
    let updated = svc
        .update_attribute_kind(&describe, &account_id())
        .unwrap()
        .unwrap();
    assert_eq!(updated.description, "How it felt");

    // Unused: deletes without force.
    svc.delete_attribute_kind("Mood", &account_id(), false)
        .unwrap();
    assert!(
        svc.get_kind_by_key("Mood", &account_id())
            .unwrap()
            .is_none()
    );

    // In use, as a scalar or a list: refused without force, purged with force.
    svc.add_attribute_kind(&add("Energy", "text", json!({})), &account_id())
        .unwrap()
        .unwrap();
    let tags = json!({"list_type": "dropdown", "values": ["a", "b"]});
    svc.add_attribute_kind(&add("Tags", "list", tags), &account_id())
        .unwrap()
        .unwrap();
    db.exec("INSERT INTO attribute (item_id, key, value_text) VALUES (1, 'Energy', 'high')")
        .await;
    db.exec(
        "INSERT INTO attribute_list_value (item_id, key, ordinal, value_text) \
         VALUES (1, 'Tags', 0, 'a'), (1, 'Tags', 1, 'b'), (2, 'Tags', 0, 'a')",
    )
    .await;
    for (key, items) in [("Energy", "1 item"), ("Tags", "2 item")] {
        let in_use = svc.delete_attribute_kind(key, &account_id(), false);
        assert!(
            matches!(&in_use, Err(AttributeServiceError::InUse(msg)) if msg.contains(items)),
            "{key}: {in_use:?}"
        );
        svc.delete_attribute_kind(key, &account_id(), true).unwrap();
        assert!(svc.get_kind_by_key(key, &account_id()).unwrap().is_none());
        assert_eq!(count_values(db, key).await, 0, "{key} values purged");
    }
    // Other keys survive a forced purge.
    assert_eq!(count_values(db, "Parent").await, 1);
    assert_eq!(count_values(db, "Status").await, 1);
}

async fn sqlite_pool() -> SqlitePool {
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
        pool.exec(statement).await;
    }
    pool
}

/// Keep the container alive for the whole test.
async fn postgres_pool() -> (PgPool, ContainerAsync<Postgres>) {
    let container = Postgres::default()
        .with_tag("16-alpine")
        .start()
        .await
        .unwrap();
    let port = container.get_host_port_ipv4(5432).await.unwrap();
    let pool: PgPool = PgPoolOptions::new()
        .max_connections(2)
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
        pool.exec(statement).await;
    }
    (pool, container)
}

#[tokio::test(flavor = "multi_thread")]
async fn system_kinds_reject_delete_and_update_on_sqlite() {
    let pool = sqlite_pool().await;
    check_system_kinds_reject_delete_and_update(&pool, get_sqlite_repos(pool.clone())).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn user_kinds_update_and_delete_on_sqlite() {
    let pool = sqlite_pool().await;
    check_user_kinds_update_and_delete(&pool, get_sqlite_repos(pool.clone())).await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "starts a PostgreSQL container; requires a running Docker daemon"]
async fn system_kinds_reject_delete_and_update_on_postgres() {
    let (pool, _container) = postgres_pool().await;
    check_system_kinds_reject_delete_and_update(&pool, get_postgres_repos(pool.clone())).await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "starts a PostgreSQL container; requires a running Docker daemon"]
async fn user_kinds_update_and_delete_on_postgres() {
    let (pool, _container) = postgres_pool().await;
    check_user_kinds_update_and_delete(&pool, get_postgres_repos(pool.clone())).await;
}

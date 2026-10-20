//! A type's required attributes must name existing attribute kinds: unknown keys are rejected on
//! create and update with nothing stored, and responses show what was persisted. Runs against the
//! real repositories and migrations. No production data.
//!
//! The PostgreSQL test starts a throwaway container with testcontainers, so it is `#[ignore]`d and
//! needs a running Docker daemon: `cargo test -p zealot-infra unknown_required -- --ignored`.
use std::sync::Arc;

use sqlx::{PgPool, SqlitePool, postgres::PgPoolOptions, sqlite::SqlitePoolOptions};
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{ContainerAsync, ImageExt, runners::AsyncRunner},
};
use zealot_app::{
    repos::ZealotRepos,
    services::item_type::{ItemTypeService, ItemTypeServiceError, KindDefaults},
};
use zealot_domain::{
    common::id::Id,
    item_type::{AddItemTypeDto, UpdateItemTypeDto},
};

use super::{postgres::get_postgres_repos, sqlite::get_sqlite_repos};

const SEED: &str = "INSERT INTO account (account_id, username, email, password) VALUES (1, 'u', 'u@example.com', 'x')";

fn account_id() -> Id {
    Id::try_from(1i64).unwrap()
}

fn service(repos: &ZealotRepos) -> ItemTypeService {
    ItemTypeService::new(&repos.item_type)
        .with_defaults(Arc::new(KindDefaults::new(&repos.attribute)))
}

fn add(name: &str, required: &[&str]) -> AddItemTypeDto {
    AddItemTypeDto {
        name: name.into(),
        description: String::new(),
        required_attributes: required.iter().map(|k| k.to_string()).collect(),
        required_defaults: None,
    }
}

fn update_required(type_id: &Id, required: &[&str]) -> UpdateItemTypeDto {
    UpdateItemTypeDto {
        type_id: i64::from(*type_id),
        name: None,
        description: None,
        required_attributes: Some(required.iter().map(|k| k.to_string()).collect()),
        required_defaults: None,
        dry_run: None,
    }
}

async fn check_unknown_required_attributes_are_rejected(repos: ZealotRepos) {
    let types = service(&repos);

    // Create: unknown keys are named, and the type is not stored.
    let err = types
        .add_item_type(
            &add("Broken", &["Priority", "Icon", "Shape"]),
            &account_id(),
        )
        .unwrap_err();
    assert!(
        matches!(&err, ItemTypeServiceError::UnknownAttributes(msg)
            if msg.contains("'Icon'") && msg.contains("'Shape'") && !msg.contains("'Priority'")),
        "{err:?}"
    );
    assert!(
        types
            .get_item_type_by_name("Broken", &account_id())
            .unwrap()
            .is_none()
    );

    // Create with a known (system) kind keeps working and reports what is stored.
    let ok = types
        .add_item_type(&add("Good", &["Priority", "Priority"]), &account_id())
        .unwrap()
        .unwrap();
    assert_eq!(ok.required_attributes, vec!["Priority".to_string()]);

    // Update: unknown keys are rejected and the stored requirements are unchanged.
    let err = types
        .update_item_type(
            &update_required(&ok.type_id, &["Priority", "Icon"]),
            &account_id(),
        )
        .unwrap_err();
    assert!(
        matches!(&err, ItemTypeServiceError::UnknownAttributes(msg) if msg.contains("'Icon'")),
        "{err:?}"
    );
    let stored = types
        .get_item_type(&ok.type_id, &account_id())
        .unwrap()
        .unwrap();
    assert_eq!(stored.required_attributes, vec!["Priority".to_string()]);

    // Update with only known keys is unchanged behaviour (dropping Priority here).
    let change = types
        .update_item_type(&update_required(&ok.type_id, &[]), &account_id())
        .unwrap()
        .unwrap();
    assert!(change.item_type.required_attributes.is_empty());
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
    sqlx::query(SEED).execute(&pool).await.unwrap();
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
    sqlx::query(SEED).execute(&pool).await.unwrap();
    (pool, container)
}

#[tokio::test(flavor = "multi_thread")]
async fn unknown_required_attributes_are_rejected_on_sqlite() {
    let pool = sqlite_pool().await;
    check_unknown_required_attributes_are_rejected(get_sqlite_repos(pool.clone())).await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "starts a PostgreSQL container; requires a running Docker daemon"]
async fn unknown_required_attributes_are_rejected_on_postgres() {
    let (pool, _container) = postgres_pool().await;
    check_unknown_required_attributes_are_rejected(get_postgres_repos(pool.clone())).await;
}

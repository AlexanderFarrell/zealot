//! An account's own type wins over a system type of the same name (Z277):
//! lookups by name, type lists and assignment by name use the account's type,
//! while other accounts keep seeing the system type.

use std::sync::Arc;

use sqlx::{postgres::PgPoolOptions, sqlite::SqlitePoolOptions};
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{ImageExt, runners::AsyncRunner},
};
use zealot_app::repos::item_type::ItemTypeRepo;
use zealot_domain::common::id::Id;

use crate::repos::{postgres::get_postgres_repos, sqlite::get_sqlite_repos};

const OWN_PLAN: i64 = 200;

const SETUP: &[&str] = &[
    "INSERT INTO account (account_id, username, email, password) VALUES
        (1, 'u', 'u@example.com', 'x'), (2, 'v', 'v@example.com', 'x')",
    "INSERT INTO item_type (type_id, name, description, account_id) VALUES (200, 'Plan', 'mine', 1)",
    "INSERT INTO item (item_id, title, content, account_id) VALUES (1, 'one', '', 1), (2, 'two', '', 2)",
];

fn id(raw: i64) -> Id {
    Id::try_from(raw).unwrap()
}

fn check(repo: &Arc<dyn ItemTypeRepo>) {
    let (owner, other) = (id(1), id(2));

    let own = repo.get_item_type_by_name("Plan", &owner).unwrap().unwrap();
    assert_eq!(i64::from(own.type_id), OWN_PLAN);
    assert!(!own.is_system);
    let system = repo.get_item_type_by_name("Plan", &other).unwrap().unwrap();
    assert!(system.is_system);
    // The system type's required attributes are untouched for everyone else.
    assert!(system.required_attributes.contains(&"Status".to_string()));

    let plans = |account: &Id| -> Vec<(i64, bool)> {
        repo.get_item_types(account)
            .unwrap()
            .into_iter()
            .filter(|t| t.name == "Plan")
            .map(|t| (i64::from(t.type_id), t.is_system))
            .collect()
    };
    assert_eq!(plans(&owner), vec![(OWN_PLAN, false)]);
    assert_eq!(plans(&other), vec![(i64::from(system.type_id), true)]);

    let summary_plans = |account: &Id| -> Vec<bool> {
        repo.get_item_type_summaries(account)
            .unwrap()
            .into_iter()
            .filter(|t| t.name == "Plan")
            .map(|t| t.is_system)
            .collect()
    };
    assert_eq!(summary_plans(&owner), vec![false]);
    assert_eq!(summary_plans(&other), vec![true]);
    // Other system types are still listed for the owner.
    assert!(
        repo.get_item_types(&owner)
            .unwrap()
            .iter()
            .any(|t| t.name == "Repeat" && t.is_system)
    );

    // Assigning "Plan" by name links only the winning type.
    repo.assign_item_types(&vec!["Plan".to_string()], &id(1), &owner)
        .unwrap();
    repo.assign_item_types(&vec!["Plan".to_string()], &id(2), &other)
        .unwrap();
    let owner_refs = repo
        .get_item_type_refs_for_items(&vec![id(1)], &owner)
        .unwrap();
    let ids: Vec<i64> = owner_refs[&id(1)]
        .iter()
        .map(|t| i64::from(t.type_id))
        .collect();
    assert_eq!(ids, vec![OWN_PLAN]);
    let other_refs = repo
        .get_item_type_refs_for_items(&vec![id(2)], &other)
        .unwrap();
    assert_eq!(other_refs[&id(2)].len(), 1);
    assert!(other_refs[&id(2)][0].is_system);
}

#[tokio::test(flavor = "multi_thread")]
async fn own_type_wins_over_system_type_on_sqlite() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("migrations/sqlite")
        .run(&pool)
        .await
        .unwrap();
    for statement in SETUP {
        sqlx::query(statement).execute(&pool).await.unwrap();
    }
    check(&get_sqlite_repos(pool).item_type);
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "starts a PostgreSQL container; requires a running Docker daemon"]
async fn own_type_wins_over_system_type_on_postgres() {
    let container = Postgres::default()
        .with_tag("16-alpine")
        .start()
        .await
        .unwrap();
    let port = container.get_host_port_ipv4(5432).await.unwrap();
    let pool = PgPoolOptions::new()
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
    for statement in SETUP {
        sqlx::query(statement).execute(&pool).await.unwrap();
    }
    check(&get_postgres_repos(pool).item_type);
}

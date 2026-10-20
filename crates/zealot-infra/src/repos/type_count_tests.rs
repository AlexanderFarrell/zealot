//! Stored per-type item counts (Z224): after every step of a mixed workload
//! the stored counts equal the counts computed from the type links, and the
//! repair recomputes them.

use sqlx::{PgPool, SqlitePool, postgres::PgPoolOptions, sqlite::SqlitePoolOptions};
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{ImageExt, runners::AsyncRunner},
};
use zealot_app::repos::item_type::ItemTypeRepo;
use zealot_domain::common::id::Id;

use crate::repos::{postgres::get_postgres_repos, sqlite::get_sqlite_repos};

const SETUP: &[&str] = &[
    "INSERT INTO account (account_id, username, email, password) VALUES
        (1, 'u', 'u@example.com', 'x'), (2, 'v', 'v@example.com', 'x')",
    "INSERT INTO item_type (type_id, name, description, account_id) VALUES
        (100, 'A', 'a', 1), (101, 'B', 'b', 1), (102, 'C', 'c', 2)",
    "INSERT INTO item (item_id, title, content, account_id) VALUES
        (1, 'one', '', 1), (2, 'two', '', 1), (3, 'three', '', 1), (4, 'four', '', 1),
        (5, 'five', '', 1), (6, 'six', '', 1), (9, 'nine', '', 2), (10, 'ten', '', 2)",
];

/// Each step changes the data; the counts are compared after every one.
const WORKLOAD: &[&str] = &[
    // Import: several links in one statement.
    "INSERT INTO item_item_type_link (item_id, type_id) VALUES
        (1, 100), (2, 100), (3, 100), (3, 101), (4, 101), (9, 102), (10, 102)",
    // Archive, un-archive, archive again (insert, then update both ways).
    "INSERT INTO attribute (item_id, key, value_int) VALUES (2, 'Archived', 1)",
    "INSERT INTO attribute (item_id, key, value_int) VALUES (3, 'Archived', 1)",
    "UPDATE attribute SET value_int = 0 WHERE item_id = 3 AND key = 'Archived'",
    "UPDATE attribute SET value_int = 1 WHERE item_id = 3 AND key = 'Archived'",
    "INSERT INTO attribute (item_id, key, value_int) VALUES (1, 'Archived', 0)",
    "UPDATE attribute SET value_int = 1 WHERE item_id = 1 AND key = 'Archived'",
    // Unrelated attributes never move a count.
    "INSERT INTO attribute (item_id, key, value_int) VALUES (4, 'Priority', 1)",
    "UPDATE attribute SET value_int = 2 WHERE item_id = 4 AND key = 'Priority'",
    // Assign an archived item to another type; assign a live one.
    "INSERT INTO item_item_type_link (item_id, type_id) VALUES (2, 101), (5, 100)",
    // Unassign an archived link and a live link.
    "DELETE FROM item_item_type_link WHERE item_id = 2 AND type_id = 100",
    "DELETE FROM item_item_type_link WHERE item_id = 4 AND type_id = 101",
    // Un-archive then delete the archive flag itself.
    "UPDATE attribute SET value_int = 0 WHERE item_id = 1 AND key = 'Archived'",
    "DELETE FROM attribute WHERE item_id = 3 AND key = 'Archived'",
    // Re-archive, then delete archived and live items with their links.
    "UPDATE attribute SET value_int = 1 WHERE item_id = 1 AND key = 'Archived'",
    "DELETE FROM item WHERE item_id = 1",
    "DELETE FROM item WHERE item_id = 5",
    // Delete a type with several linked items (links cascade), then a whole account.
    "INSERT INTO item_item_type_link (item_id, type_id) VALUES (4, 101), (6, 101)",
    "DELETE FROM item_type WHERE type_id = 101",
    "DELETE FROM account WHERE account_id = 2",
];

const COMPUTED: &str = "SELECT CAST(i.account_id AS BIGINT), CAST(l.type_id AS BIGINT),
        CAST(COUNT(*) AS BIGINT), CAST(COUNT(a.item_id) AS BIGINT)
     FROM item_item_type_link l
     JOIN item i ON i.item_id = l.item_id
     LEFT JOIN attribute a ON a.item_id = i.item_id AND a.key = 'Archived' AND a.value_int = 1
     GROUP BY i.account_id, l.type_id ORDER BY 1, 2";

/// Emptied counters stay as zero rows; they are the same as no row.
const STORED: &str = "SELECT CAST(account_id AS BIGINT), CAST(type_id AS BIGINT),
        CAST(total_count AS BIGINT), CAST(archived_count AS BIGINT)
     FROM item_type_count WHERE total_count <> 0 OR archived_count <> 0 ORDER BY 1, 2";

type Counts = Vec<(i64, i64, i64, i64)>;

fn id(raw: i64) -> Id {
    Id::try_from(raw).unwrap()
}

async fn sqlite_counts(pool: &SqlitePool, sql: &str) -> Counts {
    sqlx::query_as(sql).fetch_all(pool).await.unwrap()
}

async fn postgres_counts(pool: &PgPool, sql: &str) -> Counts {
    sqlx::query_as(sql).fetch_all(pool).await.unwrap()
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
    pool
}

async fn postgres() -> (PgPool, impl Drop) {
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
    (pool, container)
}

#[tokio::test(flavor = "multi_thread")]
async fn stored_counts_match_computed_counts_through_a_mixed_workload_on_sqlite() {
    let pool = sqlite().await;
    for statement in SETUP {
        sqlx::query(statement).execute(&pool).await.unwrap();
    }
    for statement in WORKLOAD {
        sqlx::query(statement).execute(&pool).await.unwrap();
        assert_eq!(
            sqlite_counts(&pool, STORED).await,
            sqlite_counts(&pool, COMPUTED).await,
            "after: {statement}"
        );
    }
    // Account 1 ends with type A holding only item 3, its archive flag removed.
    assert_eq!(sqlite_counts(&pool, STORED).await, vec![(1, 100, 1, 0)]);
}

#[tokio::test(flavor = "multi_thread")]
async fn migrating_a_populated_database_backfills_the_counts() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    // Everything before this ticket's migration, then data, then the migration.
    let migrator = sqlx::migrate!("migrations/sqlite");
    let before: Vec<_> = migrator
        .iter()
        .filter(|m| m.version < 22)
        .cloned()
        .collect();
    let older = sqlx::migrate::Migrator {
        migrations: before.into(),
        ..sqlx::migrate::Migrator::DEFAULT
    };
    older.run(&pool).await.unwrap();
    for statement in SETUP.iter().chain(&WORKLOAD[..7]) {
        sqlx::query(statement).execute(&pool).await.unwrap();
    }
    migrator.run(&pool).await.unwrap();
    assert_eq!(
        sqlite_counts(&pool, STORED).await,
        sqlite_counts(&pool, COMPUTED).await
    );
    assert!(!sqlite_counts(&pool, STORED).await.is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn summary_reads_stored_counts_and_repair_recomputes_them_on_sqlite() {
    let pool = sqlite().await;
    for statement in SETUP.iter().chain(&WORKLOAD[..8]) {
        sqlx::query(statement).execute(&pool).await.unwrap();
    }
    let repo = get_sqlite_repos(pool.clone()).item_type;
    check_summary_and_repair(
        &repo,
        || sqlite_counts(&pool, STORED),
        || sqlite_counts(&pool, COMPUTED),
        || async {
            sqlx::query("UPDATE item_type_count SET total_count = 99 WHERE account_id = 1")
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM item_type_count WHERE type_id = 101")
                .execute(&pool)
                .await
                .unwrap();
        },
    )
    .await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "starts a PostgreSQL container; requires a running Docker daemon"]
async fn stored_counts_match_computed_counts_through_a_mixed_workload_on_postgres() {
    let (pool, _container) = postgres().await;
    for statement in SETUP {
        sqlx::query(statement).execute(&pool).await.unwrap();
    }
    for statement in WORKLOAD {
        sqlx::query(statement).execute(&pool).await.unwrap();
        assert_eq!(
            postgres_counts(&pool, STORED).await,
            postgres_counts(&pool, COMPUTED).await,
            "after: {statement}"
        );
    }
    assert_eq!(postgres_counts(&pool, STORED).await, vec![(1, 100, 1, 0)]);
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "starts a PostgreSQL container; requires a running Docker daemon"]
async fn summary_reads_stored_counts_and_repair_recomputes_them_on_postgres() {
    let (pool, _container) = postgres().await;
    for statement in SETUP.iter().chain(&WORKLOAD[..8]) {
        sqlx::query(statement).execute(&pool).await.unwrap();
    }
    let repo = get_postgres_repos(pool.clone()).item_type;
    check_summary_and_repair(
        &repo,
        || postgres_counts(&pool, STORED),
        || postgres_counts(&pool, COMPUTED),
        || async {
            sqlx::query("UPDATE item_type_count SET total_count = 99 WHERE account_id = 1")
                .execute(&pool)
                .await
                .unwrap();
            sqlx::query("DELETE FROM item_type_count WHERE type_id = 101")
                .execute(&pool)
                .await
                .unwrap();
        },
    )
    .await;
}

/// After setup + WORKLOAD[..8]: type A holds items 1, 2, 3 (all archived) and
/// type B items 3 (archived) and 4.
async fn check_summary_and_repair<F1, F2, F3, Fut1, Fut2, Fut3>(
    repo: &std::sync::Arc<dyn ItemTypeRepo>,
    stored: F1,
    computed: F2,
    corrupt: F3,
) where
    F1: Fn() -> Fut1,
    F2: Fn() -> Fut2,
    F3: Fn() -> Fut3,
    Fut1: std::future::Future<Output = Counts>,
    Fut2: std::future::Future<Output = Counts>,
    Fut3: std::future::Future<Output = ()>,
{
    let counts = |include_archived: bool| {
        repo.get_item_type_summaries_with_archived(&id(1), include_archived)
            .unwrap()
            .into_iter()
            .map(|s| (s.name, s.item_count))
            .collect::<Vec<_>>()
    };
    let with = counts(true);
    let without = counts(false);
    for (name, all, live) in [("A", 3, 0), ("B", 2, 1)] {
        assert!(with.contains(&(name.to_string(), all)), "{with:?}");
        assert!(without.contains(&(name.to_string(), live)), "{without:?}");
    }
    // The other account's type is not counted.
    assert!(!with.iter().any(|(name, _)| name == "C"));

    corrupt().await;
    assert_ne!(stored().await, computed().await);
    repo.recompute_item_type_counts(&id(1)).unwrap();
    assert_eq!(stored().await, computed().await);
    assert_eq!(counts(true), with);
}

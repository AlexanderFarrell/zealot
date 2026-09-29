//! `query_item_ids` (type, subtree, filter, sort, paging, total) against real SQLite and PostgreSQL.
//!
//! The SQLite test runs by default. The PostgreSQL test starts a throwaway container with
//! testcontainers, so it needs a running Docker daemon and is ignored by default:
//! `cargo test -p zealot-infra -- --ignored`.

use serde_json::json;
use sqlx::{PgPool, SqlitePool, postgres::PgPoolOptions, sqlite::SqlitePoolOptions};
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{ImageExt, runners::AsyncRunner},
};
use zealot_app::repos::item_attribute_value::{ItemAttributeValueRepo, ItemQuery, ItemSort};
use zealot_domain::{
    attribute::{AttributeFilter, AttributeFilterOp, AttributeListMode},
    common::id::Id,
};

use crate::repos::{
    postgres::item_attribute_value_postgres::ItemAttributeValuePostgresRepo,
    sqlite::item_attribute_value_sqlite::ItemAttributeValueSqliteRepo,
};

/// Plain SQL valid on both backends. Tree (Parent links, child -> parent):
/// Zealot(1) > Epic(2) > Z001(3), Z002(4), Z003(9); Z010(5) has parents 1 and 2;
/// ZS001(6) is under Spindle(7); Book(8) is unparented; 1 is also a child of 3 (a cycle).
const SEED: &[&str] = &[
    "INSERT INTO item_type (name, description, account_id) VALUES ('Ticket', 't', 1), ('Epic', 'e', 1), ('Project', 'p', 1), ('Book', 'b', 1)",
    "INSERT INTO attribute_kind (account_id, key, description, base_type, is_system, config) VALUES (1, 'TicketID', '', 'text', false, '{}')",
    "INSERT INTO item (item_id, title, content, account_id) VALUES
        (1, 'Zealot', '', 1), (2, 'Z-E1 Epic', '', 1), (3, 'Alpha', '', 1), (4, 'Bravo', '', 1),
        (5, 'Charlie', '', 1), (6, 'Delta', '', 1), (7, 'Spindle', '', 1), (8, 'Some Book', '', 1),
        (9, 'Echo', '', 1)",
    "INSERT INTO item_item_type_link (item_id, type_id) SELECT i, t.type_id FROM item_type t JOIN
        (SELECT 1 AS i, 'Project' AS n UNION ALL SELECT 7, 'Project' UNION ALL SELECT 2, 'Epic'
         UNION ALL SELECT 3, 'Ticket' UNION ALL SELECT 4, 'Ticket' UNION ALL SELECT 5, 'Ticket'
         UNION ALL SELECT 6, 'Ticket' UNION ALL SELECT 9, 'Ticket' UNION ALL SELECT 8, 'Book') m
        ON m.n = t.name",
    "INSERT INTO item_item_link (first_item_id, second_item_id, relationship) VALUES
        (2, 1, 'parent'), (3, 2, 'parent'), (4, 2, 'parent'), (5, 1, 'parent'), (5, 2, 'parent'),
        (9, 2, 'parent'), (6, 7, 'parent'), (1, 3, 'parent')",
    "INSERT INTO attribute (item_id, key, value_text) VALUES
        (3, 'Status', 'Working'), (4, 'Status', 'To Do'), (5, 'Status', 'Working'),
        (6, 'Status', 'Working'), (8, 'Status', 'Working'), (9, 'Status', 'Working'),
        (3, 'TicketID', 'Z001'), (4, 'TicketID', 'Z002'), (5, 'TicketID', 'Z010'),
        (6, 'TicketID', 'ZS001'), (9, 'TicketID', 'Z003')",
    "INSERT INTO attribute (item_id, key, value_int) VALUES (3, 'Priority', 5), (4, 'Priority', 8), (5, 'Priority', 2)",
];

fn id(raw: i64) -> Id {
    Id::try_from(raw).unwrap()
}

fn working() -> AttributeFilter {
    AttributeFilter {
        key: "Status".to_string(),
        op: AttributeFilterOp::Equal,
        value: json!("Working"),
        list_mode: AttributeListMode::Any,
    }
}

fn query() -> ItemQuery {
    ItemQuery {
        types: vec![],
        all_types: false,
        under: None,
        filters: vec![],
        sort: None,
        limit: 50,
        offset: 0,
    }
}

fn sort(key: &str, descending: bool) -> Option<ItemSort> {
    Some(ItemSort {
        key: key.to_string(),
        descending,
    })
}

fn run(repo: &dyn ItemAttributeValueRepo, q: &ItemQuery) -> (Vec<i64>, i64) {
    let (ids, total) = repo.query_item_ids(q, &id(1)).unwrap();
    (ids.into_iter().map(i64::from).collect(), total)
}

fn check_query(repo: &dyn ItemAttributeValueRepo) {
    // Working Tickets under Zealot, sorted by TicketID; ZS001 (other project), the Book, and
    // the root itself are excluded even though a Parent cycle leads back to it.
    let mut q = query();
    q.types = vec!["Ticket".into()];
    q.under = Some(id(1));
    q.filters = vec![working()];
    q.sort = sort("TicketID", false);
    assert_eq!(run(repo, &q), (vec![3, 9, 5], 3));

    // Without the subtree, every Working Ticket matches, still only Tickets.
    q.under = None;
    assert_eq!(run(repo, &q), (vec![3, 9, 5, 6], 4));

    // Paging: total counts all matches, the page holds a slice.
    q.sort = sort("TicketID", true);
    q.limit = 2;
    q.offset = 1;
    assert_eq!(run(repo, &q), (vec![5, 9], 4));

    // Type modes.
    let mut q = query();
    q.types = vec!["Ticket".into(), "Epic".into()];
    q.under = Some(id(1));
    assert_eq!(run(repo, &q), (vec![2, 3, 4, 5, 9], 5));
    q.all_types = true;
    assert_eq!(run(repo, &q), (vec![], 0));

    // Subtree alone, excluding the root, terminating on the cycle.
    let mut q = query();
    q.under = Some(id(2));
    assert_eq!(run(repo, &q), (vec![1, 3, 4, 5, 9], 5));

    // Numeric sort puts items without the attribute last in both directions.
    let mut q = query();
    q.types = vec!["Ticket".into()];
    q.under = Some(id(1));
    q.sort = sort("Priority", false);
    assert_eq!(run(repo, &q).0, [5, 3, 4, 9]);
    q.sort = sort("Priority", true);
    assert_eq!(run(repo, &q).0, [4, 3, 5, 9]);

    // Built-in sort keys.
    q.sort = sort("title", true);
    assert_eq!(run(repo, &q).0, [9, 5, 4, 3]);
    q.sort = sort("created", false);
    assert_eq!(run(repo, &q).1, 4);
    q.sort = sort("updated", true);
    assert_eq!(run(repo, &q).1, 4);

    // Unknown sort keys are rejected.
    q.sort = sort("Nope", false);
    assert!(repo.query_item_ids(&q, &id(1)).is_err());
}

async fn seed_sqlite() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("migrations/sqlite")
        .run(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO account (account_id, username, email, password) VALUES (1, 'u', 'u@example.com', 'x')")
        .execute(&pool)
        .await
        .unwrap();
    for statement in SEED {
        sqlx::query(statement).execute(&pool).await.unwrap();
    }
    pool
}

async fn seed_postgres(pool: &PgPool) {
    sqlx::migrate!("migrations/postgres")
        .run(pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO account (account_id, username, email, password) VALUES (1, 'u', 'u@example.com', 'x')")
        .execute(pool)
        .await
        .unwrap();
    for statement in SEED {
        sqlx::query(statement).execute(pool).await.unwrap();
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn query_item_ids_on_sqlite() {
    check_query(&ItemAttributeValueSqliteRepo::new(seed_sqlite().await));
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "starts a PostgreSQL container; requires a running Docker daemon"]
async fn query_item_ids_on_postgres() {
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
    seed_postgres(&pool).await;
    check_query(&ItemAttributeValuePostgresRepo::new(pool));
}

//! Repositories that share SQL text on one connection (Z199).
//!
//! sqlx caches prepared statements per connection, keyed by SQL text. When two
//! call sites run the same SQL text with differently typed binds, the second
//! one reuses the first one's parameter types; on PostgreSQL an `i64` sent for a
//! cached `int4` parameter fails with "incorrect binary data format in bind
//! parameter 1". Each check runs the first call site, then the second on the
//! same connection: a habit status before `add_comment`, and a new item type
//! with required attributes before adding attribute kinds to a type.
//!
//! The PostgreSQL test starts a throwaway container with testcontainers, so it
//! needs a running Docker daemon and is ignored by default:
//! `cargo test -p zealot-infra -- --ignored`.

use sqlx::{PgPool, SqlitePool, postgres::PgPoolOptions, sqlite::SqlitePoolOptions};
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{ContainerAsync, ImageExt, runners::AsyncRunner},
};
use zealot_app::repos::{comment::CommentRepo, item_type::ItemTypeRepo, repeat::RepeatRepo};
use zealot_domain::{
    account::Account,
    comment::AddCommentDto,
    common::{email::Email, id::Id},
    item_type::AddItemTypeDto,
    repeat::UpdateRepeatEntryDto,
};

use crate::repos::{
    postgres::{
        comment_postgres::CommentPostgresRepo, item_type_postgres::ItemTypePostgresRepo,
        repeat_postgres::RepeatPostgresRepo,
    },
    sqlite::{
        comment_sqlite::CommentSqliteRepo, item_type_sqlite::ItemTypeSqliteRepo,
        repeat_sqlite::RepeatSqliteRepo,
    },
};

const SEED: &[&str] = &[
    "INSERT INTO account (account_id, username, email, password) VALUES (1, 'u', 'u@example.com', 'x')",
    "INSERT INTO item (item_id, title, content, account_id) VALUES (1, 'Habit', '', 1), (2, 'Ticket', '', 1)",
];

fn account() -> Account {
    Account {
        account_id: Id::try_from(1).unwrap(),
        username: "u".to_string(),
        email: Email::try_from("u@example.com".to_string()).unwrap(),
        given_name: String::new(),
        surname: String::new(),
        settings: serde_json::json!({}),
        has_api_key: false,
    }
}

fn comment(item_id: i64, timestamp: &str, content: &str) -> AddCommentDto {
    AddCommentDto {
        item_id,
        timestamp: timestamp.to_string(),
        content: content.to_string(),
    }
}

/// Sets a habit status, then adds comments on the same connection, with and
/// without an explicit timestamp, on the habit and on another item.
fn check_comment_after_habit_status(comments: &dyn CommentRepo, repeats: &dyn RepeatRepo) {
    let account = account();
    repeats
        .set_status(
            &UpdateRepeatEntryDto {
                item_id: 1,
                date: "2026-10-03".to_string(),
                status: Some("Complete".to_string()),
                comment: None,
            },
            &account,
        )
        .unwrap();

    let first = comments
        .add_comment(
            &comment(2, "2026-10-03 11:22:00", "with timestamp"),
            &account.account_id,
        )
        .unwrap()
        .expect("item 2 exists");
    assert_eq!(i64::from(first.item_id), 2);
    assert_eq!(first.content, "with timestamp");

    let second = comments
        .add_comment(
            &comment(1, "2026-10-03 11:30:00", "on the habit"),
            &account.account_id,
        )
        .unwrap()
        .expect("item 1 exists");
    assert_eq!(i64::from(second.item_id), 1);

    let read_back = comments
        .get_for_item(&Id::try_from(2).unwrap(), &account.account_id)
        .unwrap();
    assert_eq!(read_back.len(), 1);
    assert_eq!(read_back[0].content, "with timestamp");

    assert!(matches!(
        comments.add_comment(
            &comment(99, "2026-10-03 11:22:00", "x"),
            &account.account_id
        ),
        Err(zealot_app::repos::common::RepoError::NotFound)
    ));
}

/// Creates an item type with a required attribute, then adds another attribute
/// kind to it on the same connection.
fn check_attr_kinds_after_new_item_type(types: &dyn ItemTypeRepo) {
    let account = account();
    let created = types
        .add_item_type(
            &AddItemTypeDto {
                name: "Ticket".to_string(),
                description: "t".to_string(),
                required_attributes: vec!["Status".to_string()],
            },
            &account.account_id,
        )
        .unwrap()
        .expect("item type created");
    types
        .add_attr_kinds_to_item_type(
            &vec!["Date".to_string()],
            &created.type_id,
            &account.account_id,
        )
        .unwrap();
    let read_back = types
        .get_item_type(&created.type_id, &account.account_id)
        .unwrap()
        .expect("item type exists");
    let mut required = read_back.required_attributes;
    required.sort();
    assert_eq!(required, vec!["Date".to_string(), "Status".to_string()]);
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
        sqlx::query(statement).execute(&pool).await.unwrap();
    }
    pool
}

/// One connection, so every repository shares its statement cache, as they
/// eventually do on the long-running server. Keep the container alive for the
/// whole test.
async fn postgres_pool() -> (PgPool, ContainerAsync<Postgres>) {
    let container = Postgres::default()
        .with_tag("16-alpine")
        .start()
        .await
        .unwrap();
    let port = container.get_host_port_ipv4(5432).await.unwrap();
    let pool: PgPool = PgPoolOptions::new()
        .max_connections(1)
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
    (pool, container)
}

#[tokio::test(flavor = "multi_thread")]
async fn comment_after_habit_status_on_sqlite() {
    let pool = sqlite_pool().await;
    check_comment_after_habit_status(
        &CommentSqliteRepo::new(pool.clone()),
        &RepeatSqliteRepo::new(pool),
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn attr_kinds_after_new_item_type_on_sqlite() {
    check_attr_kinds_after_new_item_type(&ItemTypeSqliteRepo::new(sqlite_pool().await));
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "starts a PostgreSQL container; requires a running Docker daemon"]
async fn comment_after_habit_status_on_postgres() {
    let (pool, _container) = postgres_pool().await;
    check_comment_after_habit_status(
        &CommentPostgresRepo::new(pool.clone()),
        &RepeatPostgresRepo::new(pool),
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "starts a PostgreSQL container; requires a running Docker daemon"]
async fn attr_kinds_after_new_item_type_on_postgres() {
    let (pool, _container) = postgres_pool().await;
    check_attr_kinds_after_new_item_type(&ItemTypePostgresRepo::new(pool));
}

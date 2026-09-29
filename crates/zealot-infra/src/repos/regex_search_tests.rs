//! Regex item search against real SQLite and PostgreSQL databases.
//!
//! The SQLite test runs by default. The PostgreSQL test starts a throwaway container with
//! testcontainers, so it needs a running Docker daemon and is ignored by default:
//! `cargo test -p zealot-infra -- --ignored`.

use regex::Regex;
use sqlx::{postgres::PgPoolOptions, sqlite::SqlitePoolOptions, PgPool, SqlitePool};
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{runners::AsyncRunner, ImageExt},
};
use zealot_app::{
    repos::{common::RepoError, item::ItemRepo},
    services::item::{scan_regex_matches, ItemServiceError, REGEX_SCAN_BATCH},
};
use zealot_domain::{
    account::Account,
    common::{email::Email, id::Id},
};

use crate::repos::{
    postgres::item_postgres::ItemPostgresRepo, sqlite::item_sqlite::ItemSqliteRepo,
};

/// (title, content, heading) rows; the fillers push these past the first scan batch.
const ROWS: [(&str, &str, Option<&str>); 4] = [
    ("AIW001 Alpha", "first", None),
    ("AIW002 Beta", "second", None),
    ("aiw003 lower", "third", None),
    ("Notes on AIW", "last", Some("Zebra")),
];
const FILLER_ROWS: i64 = REGEX_SCAN_BATCH * 2;

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

fn scan<T>(
    limit: i64,
    offset: i64,
    pattern: &str,
    fetch: impl Fn(i64) -> Result<Vec<T>, RepoError>,
    text: impl Fn(&T) -> String,
) -> Vec<T> {
    let re = Regex::new(&format!("(?i){pattern}")).unwrap();
    scan_regex_matches(
        limit,
        offset,
        |scan_offset| fetch(scan_offset).map_err(ItemServiceError::Repo),
        |row| re.is_match(&text(row)),
    )
    .unwrap()
}

fn sorted(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values
}

fn titles(repo: &dyn ItemRepo, pattern: &str, limit: i64, offset: i64) -> Vec<String> {
    let account = account();
    scan(
        limit,
        offset,
        pattern,
        |o| repo.search_items_by_title("", REGEX_SCAN_BATCH, o, &account),
        |item| item.title.clone(),
    )
    .into_iter()
    .map(|item| item.title)
    .collect()
}

fn check_regex_search(repo: &dyn ItemRepo) {
    let account = account();
    let aiw = ["AIW001 Alpha", "AIW002 Beta", "aiw003 lower"];

    // Anchors and character classes match without their literal text appearing in the title.
    assert_eq!(sorted(titles(repo, "^AIW", 50, 0)), aiw);
    assert_eq!(sorted(titles(repo, "AIW00[0-9]", 50, 0)), aiw);
    assert_eq!(titles(repo, "^(Notes|nothing)", 50, 0), ["Notes on AIW"]);
    assert_eq!(titles(repo, "^aiw00[12]", 50, 0).len(), 2);

    // Paging is over matches, and the matches sit beyond the first scan batch.
    let first = titles(repo, "^AIW", 2, 0);
    let second = titles(repo, "^AIW", 2, 2);
    assert_eq!((first.len(), second.len()), (2, 1));
    assert_eq!(sorted(first.into_iter().chain(second).collect()), aiw);
    assert!(titles(repo, "^AIW", 2, 3).is_empty());

    // Content scope.
    let content = scan(
        50,
        0,
        "^(second|third)$",
        |o| repo.search_items_by_content("", REGEX_SCAN_BATCH, o, &account),
        |item| item.content.clone(),
    );
    assert_eq!(content.len(), 2);

    // Heading scope.
    let headings = scan(
        50,
        0,
        "^z[a-z]+$",
        |o| repo.search_items_by_heading("", REGEX_SCAN_BATCH, o, &account),
        |(_, heading)| heading.clone(),
    );
    assert_eq!(headings.len(), 1);
    assert_eq!(headings[0].0.title, "Notes on AIW");
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
    sqlx::query(
        "WITH RECURSIVE n(i) AS (SELECT 1 UNION ALL SELECT i + 1 FROM n WHERE i < ?)
         INSERT INTO item (title, content, account_id)
         SELECT 'Filler ' || i, 'filler body ' || i, 1 FROM n",
    )
    .bind(FILLER_ROWS)
    .execute(&pool)
    .await
    .unwrap();
    for (title, content, heading) in ROWS {
        let item_id: i64 = sqlx::query_scalar(
            "INSERT INTO item (title, content, account_id) VALUES (?, ?, 1) RETURNING item_id",
        )
        .bind(title)
        .bind(content)
        .fetch_one(&pool)
        .await
        .unwrap();
        if let Some(heading) = heading {
            sqlx::query(
                "INSERT INTO item_heading (item_id, level, ordinal, text) VALUES (?, 1, 0, ?)",
            )
            .bind(item_id)
            .bind(heading)
            .execute(&pool)
            .await
            .unwrap();
        }
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
    sqlx::query(
        "INSERT INTO item (title, content, account_id)
         SELECT 'Filler ' || i, 'filler body ' || i, 1 FROM generate_series(1, $1::int) i",
    )
    .bind(FILLER_ROWS)
    .execute(pool)
    .await
    .unwrap();
    for (title, content, heading) in ROWS {
        let item_id: i32 = sqlx::query_scalar(
            "INSERT INTO item (title, content, account_id) VALUES ($1, $2, 1) RETURNING item_id",
        )
        .bind(title)
        .bind(content)
        .fetch_one(pool)
        .await
        .unwrap();
        if let Some(heading) = heading {
            sqlx::query(
                "INSERT INTO item_heading (item_id, level, ordinal, text) VALUES ($1, 1, 0, $2)",
            )
            .bind(item_id)
            .bind(heading)
            .execute(pool)
            .await
            .unwrap();
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn regex_search_on_sqlite() {
    check_regex_search(&ItemSqliteRepo::new(seed_sqlite().await));
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "starts a PostgreSQL container; requires a running Docker daemon"]
async fn regex_search_on_postgres() {
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
    check_regex_search(&ItemPostgresRepo::new(pool));
}

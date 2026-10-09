//! Behavior above SQLite's bound-variable limit (32,766) and incremental
//! wikilink resolution (Z223), through `ItemService` on a real in-memory
//! SQLite database seeded with set-based SQL.

use std::sync::Arc;

use sqlx::{PgPool, SqlitePool, postgres::PgPoolOptions, sqlite::SqlitePoolOptions};
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{ImageExt, runners::AsyncRunner},
};
use zealot_app::{
    ports::events::{EventPort, NoopEventPort},
    repos::{item::ItemRepo, item_link::ItemLinkRepo},
    services::item::ItemService,
};
use zealot_domain::{
    account::Account,
    common::{email::Email, id::Id},
    item::UpdateItemDto,
};

use crate::repos::{postgres::get_postgres_repos, sqlite::get_sqlite_repos};

/// More than SQLite's 32,766 bound-variable cap.
const BIG: i64 = 40_000;

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

fn id(raw: i64) -> Id {
    Id::try_from(raw).unwrap()
}

/// `BIG` items titled `Item <n>`, all of type `Bulk`; every tenth item links
/// to the item ten places on with `[[Item <n+10>]]` (rows seeded to match).
async fn seed_big() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("migrations/sqlite").run(&pool).await.unwrap();
    for statement in [
        "INSERT INTO account (account_id, username, email, password) VALUES (1, 'u', 'u@example.com', 'x')".to_string(),
        "INSERT INTO item_type (type_id, name, description, account_id) VALUES (100, 'Bulk', 'bulk', 1)".to_string(),
        format!(
            "WITH RECURSIVE n(i) AS (SELECT 1 UNION ALL SELECT i + 1 FROM n WHERE i < {BIG})
             INSERT INTO item (item_id, title, content, account_id)
             SELECT i, 'Item ' || i,
                    CASE WHEN i % 10 = 0 THEN 'see [[Item ' || (i + 10) || ']]' ELSE '' END, 1
             FROM n"
        ),
        "INSERT INTO item_item_type_link (item_id, type_id) SELECT item_id, 100 FROM item".to_string(),
        "INSERT INTO item_item_link (first_item_id, second_item_id, relationship)
         SELECT item_id, item_id + 10, 'wikilink' FROM item WHERE item_id % 10 = 0 AND item_id + 10 <= 40000".to_string(),
    ] {
        sqlx::query(&statement).execute(&pool).await.unwrap();
    }
    pool
}

fn service(pool: SqlitePool) -> (ItemService, Arc<dyn ItemLinkRepo>, Arc<dyn ItemRepo>) {
    let repos = get_sqlite_repos(pool);
    let events: Arc<dyn EventPort> = Arc::new(NoopEventPort);
    let svc = ItemService::new(
        &repos.item,
        &repos.item_attribute_value,
        &repos.item_external_link,
        &repos.item_heading,
        &repos.item_link,
        &repos.item_type,
        &repos.attribute,
        &events,
    );
    (svc, repos.item_link.clone(), repos.item.clone())
}

fn rename(svc: &ItemService, item_id: i64, title: &str) {
    svc.update_item(
        &id(item_id),
        &UpdateItemDto {
            item_id,
            title: Some(title.to_string()),
            content: None,
            attributes: None,
            links: None,
            expected_version: None,
        },
        &account(),
    )
    .unwrap()
    .unwrap();
}

fn wikilink_sources(links: &Arc<dyn ItemLinkRepo>, target: i64) -> Vec<i64> {
    let mut ids: Vec<i64> = links
        .get_source_item_ids(&id(target), "wikilink", &id(1))
        .unwrap()
        .into_iter()
        .map(i64::from)
        .collect();
    ids.sort();
    ids
}

#[tokio::test(flavor = "multi_thread")]
async fn items_of_a_large_type_are_listed_above_the_bound_variable_limit() {
    let (svc, _, _) = service(seed_big().await);
    let items = svc
        .get_items_by_type_with_archived("Bulk", false, &account())
        .unwrap();
    assert_eq!(items.len() as i64, BIG);
}

#[tokio::test(flavor = "multi_thread")]
async fn items_load_by_any_number_of_ids() {
    let (_, _, items) = service(seed_big().await);
    let ids: Vec<Id> = (1..=BIG).map(id).collect();
    let found = items.get_items_by_ids(&ids, &account()).unwrap();
    assert_eq!(found.len() as i64, BIG);
    assert_eq!(i64::from(found[0].item_id), 1);
    assert_eq!(i64::from(found[(BIG - 1) as usize].item_id), BIG);
}

async fn seed_refs(pool: &SqlitePool) {
    for statement in [
        "INSERT INTO item_wikilink_ref (item_id, target_title)
         SELECT item_id, 'Item ' || (item_id + 10) FROM item WHERE item_id % 10 = 0",
        "INSERT INTO wikilink_ref_state (account_id) VALUES (1)",
    ] {
        sqlx::query(statement).execute(pool).await.unwrap();
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn rename_above_the_bound_variable_limit_re_resolves_only_affected_links() {
    let pool = seed_big().await;
    seed_refs(&pool).await;
    let (svc, links, _) = service(pool);

    let started = std::time::Instant::now();
    rename(&svc, 20, "Item twenty");
    rename(&svc, 21, "Item 20");
    // A full rebuild of 40,000 items takes tens of seconds in a debug build.
    assert!(
        started.elapsed() < std::time::Duration::from_secs(5),
        "two renames took {:?}",
        started.elapsed()
    );

    assert_eq!(wikilink_sources(&links, 20), Vec::<i64>::new());
    assert_eq!(wikilink_sources(&links, 21), vec![10]);
}

/// Every wikilink row, as (source, target), sorted.
async fn wikilinks(pool: &SqlitePool) -> Vec<(i64, i64)> {
    sqlx::query_as::<_, (i64, i64)>(
        "SELECT first_item_id, second_item_id FROM item_item_link
         WHERE relationship = 'wikilink' ORDER BY 1, 2",
    )
    .fetch_all(pool)
    .await
    .unwrap()
}

async fn refs(pool: &SqlitePool) -> Vec<(i64, String)> {
    sqlx::query_as::<_, (i64, String)>(
        "SELECT item_id, target_title FROM item_wikilink_ref ORDER BY 1, 2",
    )
    .fetch_all(pool)
    .await
    .unwrap()
}

/// Small fixture covering dangling links, shared titles, `[[type:Title]]`,
/// self links and links to several titles in one body.
async fn seed_small() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("migrations/sqlite").run(&pool).await.unwrap();
    for statement in [
        "INSERT INTO account (account_id, username, email, password) VALUES (1, 'u', 'u@example.com', 'x')",
        "INSERT INTO item (item_id, title, content, account_id) VALUES
            (1, 'Alpha', 'to [[Beta]] and [[Gamma]] and [[Missing]]', 1),
            (2, 'Beta', 'self [[Beta]] and [[Alpha]]', 1),
            (3, 'Gamma', '', 1),
            (4, 'Delta', 'typed [[Ticket:Gamma]] and [[Shared]]', 1),
            (5, 'Shared', '', 1),
            (6, 'Shared', '', 1),
            (7, 'Echo', '[[Delta]] [[Delta]] [[ Alpha ]]', 1)",
    ] {
        sqlx::query(statement).execute(&pool).await.unwrap();
    }
    pool
}

#[tokio::test(flavor = "multi_thread")]
async fn incremental_renames_match_a_full_rebuild() {
    let pool = seed_small().await;
    let (svc, _, _) = service(pool.clone());
    svc.rebuild_wiki_links_for_account(&account()).unwrap();
    assert!(wikilinks(&pool).await.contains(&(1, 2)));

    // (item, new title): moves a link away, onto another item, onto a shared
    // title, back, to the same title, and onto a still-missing title.
    for (item, title) in [
        (2, "Beta2"),
        (3, "Missing"),
        (5, "Gamma"),
        (6, "Alpha"),
        (2, "Beta"),
        (4, "Delta"),
        (1, "Gamma"),
        (3, "Gamma3"),
        (7, "Echo2"),
    ] {
        rename(&svc, item, title);
        let incremental = wikilinks(&pool).await;
        let incremental_refs = refs(&pool).await;

        svc.rebuild_wiki_links_for_account(&account()).unwrap();
        assert_eq!(
            incremental,
            wikilinks(&pool).await,
            "links differ after renaming {item} to {title}"
        );
        assert_eq!(incremental_refs, refs(&pool).await);
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn refs_follow_content_edits_and_record_unresolved_titles() {
    let pool = seed_small().await;
    let (svc, _, _) = service(pool.clone());
    svc.rebuild_wiki_links_for_account(&account()).unwrap();
    assert!(refs(&pool).await.contains(&(1, "Missing".to_string())));
    assert!(refs(&pool).await.contains(&(7, "Alpha".to_string())));

    svc.update_item(
        &id(1),
        &UpdateItemDto {
            item_id: 1,
            title: None,
            content: Some("only [[Gamma]]".to_string()),
            attributes: None,
            links: None,
            expected_version: None,
        },
        &account(),
    )
    .unwrap();
    assert!(!refs(&pool).await.contains(&(1, "Missing".to_string())));
    assert!(refs(&pool).await.contains(&(1, "Gamma".to_string())));
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "starts a PostgreSQL container; requires a running Docker daemon"]
async fn incremental_renames_match_a_full_rebuild_on_postgres() {
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
    sqlx::migrate!("migrations/postgres").run(&pool).await.unwrap();
    for statement in [
        "INSERT INTO account (account_id, username, email, password) VALUES (1, 'u', 'u@example.com', 'x')",
        "INSERT INTO item (item_id, title, content, account_id) VALUES
            (1, 'Alpha', 'to [[Beta]] and [[Gamma]] and [[Missing]]', 1),
            (2, 'Beta', 'self [[Beta]] and [[Alpha]]', 1),
            (3, 'Gamma', '', 1),
            (4, 'Delta', 'typed [[Ticket:Gamma]] and [[Shared]]', 1),
            (5, 'Shared', '', 1),
            (6, 'Shared', '', 1),
            (7, 'Echo', '[[Delta]] [[Delta]] [[ Alpha ]]', 1)",
    ] {
        sqlx::query(statement).execute(&pool).await.unwrap();
    }

    let repos = get_postgres_repos(pool.clone());
    let events: Arc<dyn EventPort> = Arc::new(NoopEventPort);
    let svc = ItemService::new(
        &repos.item,
        &repos.item_attribute_value,
        &repos.item_external_link,
        &repos.item_heading,
        &repos.item_link,
        &repos.item_type,
        &repos.attribute,
        &events,
    );
    svc.rebuild_wiki_links_for_account(&account()).unwrap();

    let links = || async {
        sqlx::query_as::<_, (i32, i32)>(
            "SELECT first_item_id, second_item_id FROM item_item_link
             WHERE relationship = 'wikilink' ORDER BY 1, 2",
        )
        .fetch_all(&pool)
        .await
        .unwrap()
    };
    let refs = || async {
        sqlx::query_as::<_, (i32, String)>(
            "SELECT item_id, target_title FROM item_wikilink_ref ORDER BY 1, 2",
        )
        .fetch_all(&pool)
        .await
        .unwrap()
    };
    assert!(links().await.contains(&(1, 2)));

    for (item, title) in [
        (2, "Beta2"),
        (3, "Missing"),
        (5, "Gamma"),
        (6, "Alpha"),
        (2, "Beta"),
        (4, "Delta"),
        (1, "Gamma"),
        (3, "Gamma3"),
        (7, "Echo2"),
    ] {
        rename(&svc, item, title);
        let incremental = (links().await, refs().await);
        svc.rebuild_wiki_links_for_account(&account()).unwrap();
        assert_eq!(
            incremental,
            (links().await, refs().await),
            "differs after renaming {item} to {title}"
        );
    }
}

//! Server-side content edits and grep (Z194) through `ItemService` on a real
//! SQLite database, plus the repository compare-and-swap on SQLite and PostgreSQL.
//!
//! The PostgreSQL test starts a throwaway container with testcontainers, so it
//! needs a running Docker daemon and is ignored by default:
//! `cargo test -p zealot-infra -- --ignored`.

use std::sync::Arc;

use sqlx::{PgPool, SqlitePool, postgres::PgPoolOptions, sqlite::SqlitePoolOptions};
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{ImageExt, runners::AsyncRunner},
};
use zealot_app::{
    ports::events::{EventPort, NoopEventPort},
    repos::item::ItemRepo,
    services::item::{ItemService, ItemServiceError},
};
use zealot_domain::{
    account::Account,
    common::{email::Email, id::Id},
    item::UpdateItemDto,
    item_content::{
        ChangedLines, ContentEdit, ContentEditDto, ContentError, GrepQueryDto, content_version,
    },
};

use crate::repos::{
    postgres::item_postgres::ItemPostgresRepo,
    sqlite::{get_sqlite_repos, item_sqlite::ItemSqliteRepo},
};

const BODY: &str = "# Goal\nShip it.\n\n## Detail\nOld sentence here.\n\n# Notes\nTODO a\nTODO b\n";

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
    seed(&pool).await;
    pool
}

async fn seed<'e, E>(executor: E)
where
    E: sqlx::Executor<'e> + Copy,
    <E as sqlx::Executor<'e>>::Database: sqlx::Database,
    for<'q> <<E as sqlx::Executor<'e>>::Database as sqlx::Database>::Arguments<'q>:
        sqlx::IntoArguments<'q, <E as sqlx::Executor<'e>>::Database>,
{
    for statement in [
        "INSERT INTO account (account_id, username, email, password) VALUES (1, 'u', 'u@example.com', 'x')",
        "INSERT INTO item (item_id, title, content, account_id) VALUES (1, 'Doc', '# Goal\nShip it.\n\n## Detail\nOld sentence here.\n\n# Notes\nTODO a\nTODO b\n', 1)",
    ] {
        sqlx::query(statement).execute(executor).await.unwrap();
    }
}

fn service(pool: SqlitePool) -> ItemService {
    let repos = get_sqlite_repos(pool);
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

fn content(svc: &ItemService) -> String {
    svc.get_item_by_id(&id(1), &account())
        .unwrap()
        .unwrap()
        .content
}

fn edit(edit: ContentEdit, expected_version: Option<&str>) -> ContentEditDto {
    ContentEditDto {
        edit,
        expected_version: expected_version.map(str::to_string),
    }
}

fn replace(old: &str, new: &str, replace_all: bool) -> ContentEdit {
    ContentEdit::Replace {
        old: old.to_string(),
        new: new.to_string(),
        replace_all,
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn edit_replaces_one_sentence_and_leaves_the_rest() {
    let svc = service(seed_sqlite().await);
    let version = content_version(BODY);

    let written = svc
        .edit_content(
            &id(1),
            &edit(
                replace("Old sentence", "New sentence", false),
                Some(&version),
            ),
            &account(),
        )
        .unwrap();

    let expected = BODY.replace("Old sentence", "New sentence");
    assert_eq!(content(&svc), expected);
    assert_eq!(written.version, content_version(&expected));
    assert_eq!(written.content_chars, expected.chars().count());
    assert_eq!(
        written.changed_lines,
        Some(ChangedLines {
            start: 5,
            removed: 1,
            added: 1
        })
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn edit_errors_leave_the_content_alone() {
    let svc = service(seed_sqlite().await);

    let missing = svc.edit_content(
        &id(1),
        &edit(replace("absent", "x", false), None),
        &account(),
    );
    assert!(matches!(
        missing,
        Err(ItemServiceError::Content(ContentError::TextNotFound))
    ));

    let ambiguous = svc.edit_content(
        &id(1),
        &edit(replace("TODO", "DONE", false), None),
        &account(),
    );
    assert!(matches!(
        ambiguous,
        Err(ItemServiceError::Content(ContentError::TextAmbiguous {
            count: 2,
            ..
        }))
    ));
    assert_eq!(content(&svc), BODY);

    let no_item = svc.edit_content(&id(99), &edit(replace("a", "b", false), None), &account());
    assert!(matches!(no_item, Err(ItemServiceError::NotFound)));

    svc.edit_content(
        &id(1),
        &edit(replace("TODO", "DONE", true), None),
        &account(),
    )
    .unwrap();
    assert_eq!(content(&svc), BODY.replace("TODO", "DONE"));
}

#[tokio::test(flavor = "multi_thread")]
async fn stale_version_is_refused_on_every_write() {
    let svc = service(seed_sqlite().await);
    let stale = content_version(BODY);
    // Someone edits the item in the UI after the agent read it.
    svc.edit_content(
        &id(1),
        &edit(
            ContentEdit::Append {
                text: "UI edit".into(),
            },
            None,
        ),
        &account(),
    )
    .unwrap();
    let after_ui = content(&svc);

    for op in [
        replace("Ship it.", "Shipped.", false),
        ContentEdit::Append {
            text: "agent".into(),
        },
        ContentEdit::ReplaceSection {
            heading: "Detail".into(),
            content: "agent".into(),
        },
    ] {
        let result = svc.edit_content(&id(1), &edit(op, Some(&stale)), &account());
        assert!(
            matches!(result, Err(ItemServiceError::Conflict(_))),
            "{result:?}"
        );
    }

    let update = UpdateItemDto {
        item_id: 1,
        title: None,
        content: Some("overwrite".into()),
        attributes: None,
        links: None,
        expected_version: Some(stale),
    };
    let result = svc.update_item(&id(1), &update, &account());
    assert!(matches!(result, Err(ItemServiceError::Conflict(_))));
    assert_eq!(content(&svc), after_ui);
}

#[tokio::test(flavor = "multi_thread")]
async fn update_with_current_version_succeeds() {
    let svc = service(seed_sqlite().await);
    let update = UpdateItemDto {
        item_id: 1,
        title: Some("Renamed".into()),
        content: Some("# New\n".into()),
        attributes: None,
        links: None,
        expected_version: Some(content_version(BODY)),
    };
    let item = svc
        .update_item(&id(1), &update, &account())
        .unwrap()
        .unwrap();
    assert_eq!(item.title, "Renamed");
    assert_eq!(item.content, "# New\n");
}

#[tokio::test(flavor = "multi_thread")]
async fn append_and_replace_section_run_on_the_server() {
    let svc = service(seed_sqlite().await);

    svc.edit_content(
        &id(1),
        &edit(
            ContentEdit::ReplaceSection {
                heading: "## Detail".into(),
                content: "Rewritten.\n".into(),
            },
            None,
        ),
        &account(),
    )
    .unwrap();
    let written = svc
        .edit_content(
            &id(1),
            &edit(
                ContentEdit::Append {
                    text: "- log".into(),
                },
                None,
            ),
            &account(),
        )
        .unwrap();

    let expected = "# Goal\nShip it.\n\n## Detail\nRewritten.\n# Notes\nTODO a\nTODO b\n- log\n";
    assert_eq!(content(&svc), expected);
    assert_eq!(
        written.changed_lines,
        Some(ChangedLines {
            start: 9,
            removed: 0,
            added: 1
        })
    );

    let missing = svc.edit_content(
        &id(1),
        &edit(
            ContentEdit::ReplaceSection {
                heading: "Nope".into(),
                content: "x".into(),
            },
            None,
        ),
        &account(),
    );
    assert!(matches!(
        missing,
        Err(ItemServiceError::Content(
            ContentError::HeadingNotFound { .. }
        ))
    ));
}

/// The compare-and-swap writes only when the content still matches.
fn check_compare_and_swap(repo: &dyn ItemRepo) {
    let account = account();
    assert!(
        !repo
            .update_content_if_unchanged(&id(1), "not the body", None, "lost", &account)
            .unwrap()
    );
    assert_eq!(
        repo.get_item_by_id(&id(1), &account)
            .unwrap()
            .unwrap()
            .content,
        BODY
    );

    assert!(
        repo.update_content_if_unchanged(&id(1), BODY, Some("T2"), "won", &account)
            .unwrap()
    );
    let item = repo.get_item_by_id(&id(1), &account).unwrap().unwrap();
    assert_eq!((item.title.as_str(), item.content.as_str()), ("T2", "won"));

    // A second writer holding the old body now loses.
    assert!(
        !repo
            .update_content_if_unchanged(&id(1), BODY, None, "late", &account)
            .unwrap()
    );
    assert!(
        !repo
            .update_content_if_unchanged(&id(99), "", None, "x", &account)
            .unwrap()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn compare_and_swap_on_sqlite() {
    check_compare_and_swap(&ItemSqliteRepo::new(seed_sqlite().await));
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "starts a PostgreSQL container; requires a running Docker daemon"]
async fn compare_and_swap_on_postgres() {
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
    seed(&pool).await;
    check_compare_and_swap(&ItemPostgresRepo::new(pool));
}

/// Zealot(10) > Epic(11) > Ticket(12), Ticket(13); Spindle(14) is another
/// project. Every body mentions Z190 except 13.
const GREP_SEED: &[&str] = &[
    "INSERT INTO item_type (name, description, account_id) VALUES ('Ticket', 't', 1)",
    "INSERT INTO item (item_id, title, content, account_id) VALUES
        (10, 'Zealot', 'Root mentions Z190', 1),
        (11, 'Epic', 'intro
see Z190 here
outro', 1),
        (12, 'Ticket A', 'a
b
fixes z190.
c', 1),
        (13, 'Ticket B', 'nothing', 1),
        (14, 'Spindle', 'Z190 elsewhere', 1)",
    "INSERT INTO item_item_type_link (item_id, type_id) SELECT i.item_id, t.type_id FROM item i, item_type t WHERE i.item_id IN (12, 13)",
    "INSERT INTO item_item_link (first_item_id, second_item_id, relationship) VALUES
        (11, 10, 'parent'), (12, 11, 'parent'), (13, 11, 'parent')",
];

async fn grep_service() -> ItemService {
    let pool = seed_sqlite().await;
    for statement in GREP_SEED {
        sqlx::query(statement).execute(&pool).await.unwrap();
    }
    service(pool)
}

fn grep(pattern: &str) -> GrepQueryDto {
    GrepQueryDto {
        pattern: pattern.to_string(),
        ..Default::default()
    }
}

fn hit_ids(svc: &ItemService, query: &GrepQueryDto) -> Vec<i64> {
    svc.grep_items(query, &account())
        .unwrap()
        .items
        .into_iter()
        .map(|hit| hit.item_id)
        .collect()
}

#[tokio::test(flavor = "multi_thread")]
async fn grep_finds_lines_across_all_items_and_a_subtree() {
    let svc = grep_service().await;

    // Everywhere, case-insensitive, in item id order.
    assert_eq!(hit_ids(&svc, &grep("Z190")), vec![10, 11, 12, 14]);

    // Only descendants of Zealot: the root and the other project are excluded.
    let mut under = grep("z190");
    under.under = Some(10);
    let page = svc.grep_items(&under, &account()).unwrap();
    assert_eq!(page.count, 2);
    assert_eq!(page.items[0].title, "Epic");
    assert_eq!(page.items[0].lines, vec!["2:see Z190 here"]);
    assert_eq!(page.items[1].lines, vec!["3:fixes z190."]);
    assert_eq!(page.items[1].match_count, 1);

    // Types narrow further; context lines come back grep-formatted.
    under.types = vec!["Ticket".to_string()];
    under.context_lines = 1;
    let page = svc.grep_items(&under, &account()).unwrap();
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].lines, vec!["2-b", "3:fixes z190.", "4-c"]);
}

#[tokio::test(flavor = "multi_thread")]
async fn grep_pages_literals_and_regexes() {
    let svc = grep_service().await;

    let mut query = grep("Z190");
    query.limit = Some(3);
    let first = svc.grep_items(&query, &account()).unwrap();
    assert_eq!(first.next_offset, Some(3));
    query.offset = 3;
    let second = svc.grep_items(&query, &account()).unwrap();
    assert_eq!(
        second.items.iter().map(|h| h.item_id).collect::<Vec<_>>(),
        vec![14]
    );
    assert_eq!(second.next_offset, None);

    // A literal pattern treats regex characters as text.
    assert_eq!(hit_ids(&svc, &grep("z190.")), vec![12]);
    let mut anchored = grep("^Z190");
    assert!(hit_ids(&svc, &anchored).is_empty());
    anchored.regex = true;
    assert_eq!(hit_ids(&svc, &anchored), vec![14]);

    let mut bad = grep("(");
    bad.regex = true;
    assert!(matches!(
        svc.grep_items(&bad, &account()),
        Err(ItemServiceError::InvalidRegex(_))
    ));
    assert!(matches!(
        svc.grep_items(&grep(""), &account()),
        Err(ItemServiceError::InvalidFilter(_))
    ));
}

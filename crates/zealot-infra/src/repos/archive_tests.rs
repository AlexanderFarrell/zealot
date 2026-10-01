//! Reversible archiving through the service and real repositories. No production data.
use super::{postgres::get_postgres_repos, sqlite::get_sqlite_repos};
use serde_json::{Value, json};
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
        item::{ItemService, ItemServiceError},
        planner::PlannerService,
    },
};
use zealot_domain::{
    account::Account,
    attribute::AttributeFilterDto,
    common::{email::Email, id::Id},
    item::{ItemDto, ItemQueryDto, SearchScope},
};

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
        settings: json!({}),
        has_api_key: false,
    }
}
fn service(repos: &ZealotRepos) -> ItemService {
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
const SEED: &[&str] = &[
    "INSERT INTO account (account_id, username, email, password) VALUES (1, 'u', 'u@example.com', 'x'), (2, 'v', 'v@example.com', 'x')",
    "INSERT INTO item (item_id, title, content, account_id) VALUES (1, 'Archive root', '# Archive body', 1), (2, 'Archive child', '[[Archive root]]', 1), (3, 'Archive live', '# Archive body', 1), (4, 'Archive private', '', 2)",
    "INSERT INTO item_item_link (first_item_id, second_item_id, relationship) VALUES (2, 1, 'parent'), (1, 2, 'parent'), (4, 1, 'parent'), (3, 1, 'wikilink'), (2, 1, 'other')",
    "INSERT INTO attribute (item_id, key, value_text) VALUES (1, 'Status', 'Working'), (1, 'Week', '2026-W40')",
    "INSERT INTO attribute (item_id, key, value_int) VALUES (1, 'Priority', 10), (1, 'Root', 1), (1, 'Month', 10), (1, 'Year', 2026)",
    "INSERT INTO item_item_type_link (item_id, type_id) SELECT 1, type_id FROM item_type WHERE name = 'Plan'",
    "INSERT INTO item_heading (item_id, level, ordinal, text) VALUES (1, 1, 0, 'Archive heading'), (3, 1, 0, 'Archive heading')",
    "INSERT INTO item_view (item_id) VALUES (1), (1), (3)",
];
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
    sqlx::query("INSERT INTO attribute (item_id, key, value_date) VALUES (1, 'Date', strftime('%s', '2026-10-01'))").execute(&pool).await.unwrap();
    pool
}
fn history(item: &zealot_domain::item::Item) -> Value {
    let mut value = serde_json::to_value(ItemDto::from(item)).unwrap();
    value.as_object_mut().unwrap().remove("archived");
    value["attributes"]
        .as_object_mut()
        .unwrap()
        .remove("Archived");
    value
}
fn verify(repos: ZealotRepos) {
    let svc = Arc::new(service(&repos));
    let user = account();
    let before = svc.get_item_by_id(&id(1), &user).unwrap().unwrap();
    assert_eq!(
        svc.set_archived(&id(4), true, true, &user)
            .unwrap_err()
            .to_string(),
        ItemServiceError::NotFound.to_string()
    );
    let changed = svc.set_archived(&id(1), true, true, &user).unwrap();
    assert_eq!(changed, vec![id(1), id(2)]); // Cycle terminates; foreign descendant is untouched.
    let archived = svc.get_item_by_id(&id(1), &user).unwrap().unwrap();
    assert!(ItemDto::from(&archived).archived);
    assert_eq!(history(&before), history(&archived));
    assert!(svc.get_items_by_title("Archive root", &user).unwrap()[0].is_archived()); // Wikilink target still resolves.
    let mut other = user.clone();
    other.account_id = id(2);
    assert!(
        !svc.get_item_by_id(&id(4), &other)
            .unwrap()
            .unwrap()
            .is_archived()
    );
    assert!(svc.get_root_items(&user).unwrap().is_empty());
    assert_eq!(
        svc.get_root_items_with_archived(true, &user).unwrap().len(),
        1
    );
    assert!(svc.get_items_by_type("Plan", &user).unwrap().is_empty());
    assert_eq!(
        svc.get_items_by_type_with_archived("Plan", true, &user)
            .unwrap()
            .len(),
        1
    );
    assert!(svc.get_children(&id(1), &user).unwrap().is_empty());
    assert_eq!(
        svc.get_children_with_archived(&id(1), true, &user)
            .unwrap()
            .len(),
        1
    );
    assert!(svc.get_related_items(&id(2), &user).unwrap().is_empty());
    assert_eq!(
        svc.get_related_items_with_archived(&id(2), true, &user)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(svc.get_backlinks(&id(1), &user).unwrap().len(), 1);
    for scope in [
        SearchScope::Title,
        SearchScope::Content,
        SearchScope::Heading,
    ] {
        for regex in [false, true] {
            let hits = svc
                .search_items("Archive", scope.clone(), regex, 1, 0, &user)
                .unwrap();
            assert_eq!(hits[0].item.item_id, id(3)); // Archived matches do not consume page slots.
            assert!(
                svc.search_items("Archive", scope.clone(), regex, 1, 1, &user)
                    .unwrap()
                    .is_empty()
            );
            assert!(
                !svc.search_items_with_archived("Archive", scope.clone(), regex, 1, 0, true, &user)
                    .unwrap()
                    .is_empty()
            );
        }
    }
    let filter = AttributeFilterDto {
        key: "Status".into(),
        op: "eq".into(),
        value: json!("Working"),
        list_mode: "any".into(),
    };
    assert!(
        svc.filter_items(&vec![filter.clone()], &user)
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        svc.filter_items_with_pagination(&vec![filter], Some(1), 0, true, &user)
            .unwrap()
            .len(),
        1
    );
    let mut q = ItemQueryDto {
        types: vec!["Plan".into()],
        limit: Some(1),
        ..Default::default()
    };
    assert_eq!(svc.query_items(&q, &user).unwrap().1, 0);
    q.include_archived = true;
    assert_eq!(svc.query_items(&q, &user).unwrap().1, 1);
    let counts = repos.item_type.get_item_type_summaries(&id(1)).unwrap();
    assert_eq!(
        counts.iter().find(|t| t.name == "Plan").unwrap().item_count,
        0
    );
    let counts = repos
        .item_type
        .get_item_type_summaries_with_archived(&id(1), true)
        .unwrap();
    assert_eq!(
        counts.iter().find(|t| t.name == "Plan").unwrap().item_count,
        1
    );
    assert_eq!(
        repos.item_view.get_most_viewed(1, &id(1)).unwrap()[0].0,
        id(3)
    );
    assert_eq!(
        repos
            .item_view
            .get_most_viewed_with_archived(1, &id(1), true)
            .unwrap()[0]
            .0,
        id(1)
    );
    let planner = PlannerService::new(&svc);
    let day = chrono::NaiveDate::from_ymd_opt(2026, 10, 1).unwrap();
    let week = zealot_domain::attribute::Week::try_from("2026-W40").unwrap();
    assert!(planner.get_for_day(&day, &user).unwrap().is_empty());
    assert!(planner.get_for_week(&week, &user).unwrap().is_empty());
    assert!(planner.get_for_month(10, 2026, &user).unwrap().is_empty());
    assert!(planner.get_for_year(2026, &user).unwrap().is_empty());
    assert_eq!(
        planner
            .get_for_day_with_archived(&day, true, &user)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        planner
            .get_for_week_with_archived(&week, true, &user)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        planner
            .get_for_month_with_archived(10, 2026, true, &user)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        planner
            .get_for_year_with_archived(2026, true, &user)
            .unwrap()
            .len(),
        1
    );
    svc.set_archived(&id(1), true, true, &user).unwrap(); // Idempotent.
    svc.set_archived(&id(1), false, true, &user).unwrap();
    assert_eq!(
        history(&before),
        history(&svc.get_item_by_id(&id(1), &user).unwrap().unwrap())
    );
    assert_eq!(svc.get_children(&id(1), &user).unwrap().len(), 1);
    assert_eq!(svc.query_items(&q, &user).unwrap().1, 1);
}
#[tokio::test(flavor = "multi_thread")]
async fn archive_sqlite_preserves_history_and_visibility() {
    verify(get_sqlite_repos(sqlite().await));
}
#[tokio::test(flavor = "multi_thread")]
async fn archive_sqlite_rolls_back_entire_subtree_on_failure() {
    let pool = sqlite().await;
    sqlx::query("CREATE TRIGGER fail_archive BEFORE INSERT ON attribute WHEN NEW.key = 'Archived' AND NEW.item_id = 2 BEGIN SELECT RAISE(ABORT, 'test failure'); END").execute(&pool).await.unwrap();
    let svc = service(&get_sqlite_repos(pool));
    assert!(svc.set_archived(&id(1), true, true, &account()).is_err());
    assert!(
        !svc.get_item_by_id(&id(1), &account())
            .unwrap()
            .unwrap()
            .is_archived()
    );
}
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires Docker"]
async fn archive_postgres_preserves_history_and_visibility() {
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
    sqlx::query(
        "INSERT INTO attribute (item_id, key, value_date) VALUES (1, 'Date', '2026-10-01')",
    )
    .execute(&pool)
    .await
    .unwrap();
    verify(get_postgres_repos(pool));
}

//! Changing a type's required attributes against existing items, through the services and the real
//! SQLite repositories: reject with counts, dry run, default backfill, atomicity, relationships,
//! the non-compliant listing, and the edit rule for items already out of compliance.
//! No production data.
use std::{collections::HashMap, sync::Arc};

use serde_json::{Value as JsonValue, json};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use zealot_app::{
    ports::events::{EventPort, NoopEventPort},
    services::{
        attribute::AttributeService,
        item::{ItemService, ItemServiceError},
        item_type::{ItemTypeService, ItemTypeServiceError, KindDefaults, RequirementDefaults},
    },
};
use zealot_domain::{
    account::Account,
    attribute::{AddAttributeKindDto, Attribute, AttributeScalar},
    common::{email::Email, id::Id},
    item::AddItemDto,
    item_type::{AddItemTypeDto, ItemType, UpdateItemTypeDto},
};

use super::sqlite::get_sqlite_repos;

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

/// Defaults by attribute key, standing in for attribute kind defaults.
#[derive(Debug, Default)]
struct MapDefaults(HashMap<String, Attribute>);

impl RequirementDefaults for MapDefaults {
    fn kind_default(&self, _: &Id, key: &str) -> Option<Attribute> {
        self.0.get(key).cloned()
    }

    fn parse_override(&self, _: &Id, key: &str, _: &JsonValue) -> Result<Attribute, String> {
        Err(format!(
            "MapDefaults does not validate type defaults ('{key}')"
        ))
    }
}

struct Fixture {
    pool: SqlitePool,
    types: ItemTypeService,
    items: ItemService,
    type_id: Id,
}

/// Account 1, items 1-3 of type "Thing" (item 3 archived) and item 4 untyped. Kinds: `Rating`
/// and `Note` (text), `Tags` (list), plus the system `Parent` (list of items).
async fn fixture(defaults: MapDefaults) -> Fixture {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("migrations/sqlite")
        .run(&pool)
        .await
        .unwrap();
    for statement in [
        "INSERT INTO account (account_id, username, email, password) VALUES (1, 'u', 'u@example.com', 'x')",
        "INSERT INTO item (item_id, title, content, account_id) VALUES (1, 'One', '', 1), (2, 'Two', '', 1), (3, 'Three', '', 1), (4, 'Four', '', 1)",
        "INSERT INTO attribute (item_id, key, value_int) VALUES (3, 'Archived', 1)",
    ] {
        sqlx::query(statement).execute(&pool).await.unwrap();
    }

    let repos = get_sqlite_repos(pool.clone());
    let attributes = AttributeService::new(&repos.attribute);
    for (key, base_type, config) in [
        ("Rating", "text", json!({})),
        ("Note", "text", json!({})),
        ("Tags", "list", json!({"list_type": "text"})),
    ] {
        attributes
            .add_attribute_kind(
                &AddAttributeKindDto {
                    key: key.into(),
                    description: String::new(),
                    base_type: base_type.into(),
                    config,
                },
                &id(1),
            )
            .unwrap()
            .unwrap();
    }

    let types = ItemTypeService::new(&repos.item_type).with_defaults(Arc::new(defaults));
    let thing = types
        .add_item_type(
            &AddItemTypeDto {
                name: "Thing".into(),
                description: String::new(),
                required_attributes: vec![],
                required_defaults: None,
            },
            &id(1),
        )
        .unwrap()
        .unwrap();
    for item in 1..=3 {
        sqlx::query("INSERT INTO item_item_type_link (item_id, type_id) VALUES (?, ?)")
            .bind(item)
            .bind(i64::from(thing.type_id))
            .execute(&pool)
            .await
            .unwrap();
    }

    let events: Arc<dyn EventPort> = Arc::new(NoopEventPort);
    let items = ItemService::new(
        &repos.item,
        &repos.item_attribute_value,
        &repos.item_external_link,
        &repos.item_heading,
        &repos.item_link,
        &repos.item_type,
        &repos.attribute,
        &events,
    );
    Fixture {
        pool,
        types,
        items,
        type_id: thing.type_id,
    }
}

impl Fixture {
    fn require(&self, keys: &[&str], dry_run: bool) -> UpdateItemTypeDto {
        UpdateItemTypeDto {
            type_id: i64::from(self.type_id),
            name: None,
            description: None,
            required_attributes: Some(keys.iter().map(|k| k.to_string()).collect()),
            required_defaults: None,
            dry_run: Some(dry_run),
        }
    }

    fn required(&self) -> Vec<String> {
        self.types
            .get_item_type(&self.type_id, &id(1))
            .unwrap()
            .unwrap()
            .required_attributes
    }

    async fn set_text(&self, item: i64, key: &str, value: &str) {
        sqlx::query("INSERT INTO attribute (item_id, key, value_text) VALUES (?, ?, ?)")
            .bind(item)
            .bind(key)
            .bind(value)
            .execute(&self.pool)
            .await
            .unwrap();
    }

    async fn count(&self, sql: &str) -> i64 {
        sqlx::query_scalar(sql).fetch_one(&self.pool).await.unwrap()
    }

    /// Put the type in the state older versions allowed: a requirement that items don't meet.
    async fn force_required(&self, key: &str) {
        sqlx::query(
            "INSERT INTO item_type_attribute_kind_link (attribute_kind_id, item_type_id)
             SELECT kind_id, ? FROM attribute_kind WHERE key = ? AND (account_id = 1 OR is_system = 1)",
        )
        .bind(i64::from(self.type_id))
        .bind(key)
        .execute(&self.pool)
        .await
        .unwrap();
    }
}

fn text(value: &str) -> Attribute {
    Attribute::Scalar(AttributeScalar::Text(value.into()))
}

#[tokio::test(flavor = "multi_thread")]
async fn requirement_all_items_already_satisfy_is_a_no_op_change() {
    let f = fixture(MapDefaults::default()).await;
    for item in 1..=3 {
        f.set_text(item, "Rating", "ok").await;
    }

    let change = f
        .types
        .update_item_type(&f.require(&["Rating"], false), &id(1))
        .unwrap()
        .unwrap();

    assert_eq!(f.required(), ["Rating"]);
    assert_eq!(change.report.added, ["Rating"]);
    assert!(change.report.backfilled.is_empty());
    assert_eq!(change.report.non_compliant.total, 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn requirement_without_default_is_rejected_with_counts_and_changes_nothing() {
    let f = fixture(MapDefaults::default()).await;
    f.set_text(1, "Rating", "ok").await;

    let err = f
        .types
        .update_item_type(&f.require(&["Rating"], false), &id(1))
        .unwrap_err();

    let ItemTypeServiceError::WouldInvalidate { message, summary } = err else {
        panic!("expected WouldInvalidate, got {err:?}");
    };
    assert_eq!(summary.total, 2);
    assert_eq!(summary.by_key["Rating"], 2);
    assert_eq!(summary.sample_ids["Rating"], [2, 3]);
    assert!(
        message.contains("2 existing item(s)") && message.contains("'Rating'"),
        "{message}"
    );
    assert!(f.required().is_empty());
    assert_eq!(
        f.count(&format!(
            "SELECT COUNT(*) FROM item_type_attribute_kind_link WHERE item_type_id = {}",
            i64::from(f.type_id)
        ))
        .await,
        0
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn dry_run_reports_offenders_without_error_or_writes() {
    let f = fixture(MapDefaults::default()).await;
    f.set_text(1, "Rating", "ok").await;

    let change = f
        .types
        .update_item_type(&f.require(&["Rating", "Note"], true), &id(1))
        .unwrap()
        .unwrap();

    assert!(change.report.dry_run);
    assert_eq!(change.report.non_compliant.by_key["Rating"], 2);
    assert_eq!(change.report.non_compliant.by_key["Note"], 3);
    assert_eq!(
        change.report.non_compliant.total, 3,
        "distinct items, not gaps"
    );
    assert!(f.required().is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn requirement_with_default_backfills_every_member_and_reports_counts() {
    let defaults = MapDefaults(HashMap::from([
        ("Rating".to_string(), text("3")),
        (
            "Tags".to_string(),
            Attribute::List(vec![
                AttributeScalar::Text("a".into()),
                AttributeScalar::Text("b".into()),
            ]),
        ),
    ]));
    let f = fixture(defaults).await;
    f.set_text(1, "Rating", "keep").await;

    let change = f
        .types
        .update_item_type(&f.require(&["Rating", "Tags"], false), &id(1))
        .unwrap()
        .unwrap();

    assert_eq!(change.report.backfilled["Rating"], 2);
    assert_eq!(change.report.backfilled["Tags"], 3);
    assert_eq!(change.report.non_compliant.total, 0);
    assert_eq!(
        f.count("SELECT COUNT(*) FROM attribute WHERE key = 'Rating'")
            .await,
        3
    );
    assert_eq!(
        f.count("SELECT COUNT(*) FROM attribute WHERE key = 'Rating' AND value_text = 'keep' AND item_id = 1").await,
        1,
        "existing values are never overwritten"
    );
    assert_eq!(
        f.count("SELECT COUNT(*) FROM attribute_list_value WHERE key = 'Tags'")
            .await,
        6
    );
    assert_eq!(
        f.count("SELECT COUNT(*) FROM attribute WHERE item_id = 4")
            .await,
        0,
        "non-members untouched"
    );
    assert!(
        f.types
            .list_non_compliant(&f.type_id, 50, 0, &id(1))
            .unwrap()
            .items
            .is_empty()
    );
}

/// Kind defaults as stored by Z45: the system `Priority` kind (default 3, set by migration) and
/// user kinds with a `config.default`, backfilled through `KindDefaults`; a kind with no default
/// still blocks the change.
#[tokio::test(flavor = "multi_thread")]
async fn kind_defaults_backfill_existing_items_and_kinds_without_one_block() {
    let mut f = fixture(MapDefaults::default()).await;
    let repos = get_sqlite_repos(f.pool.clone());
    f.types = ItemTypeService::new(&repos.item_type)
        .with_defaults(Arc::new(KindDefaults::new(&repos.attribute)));
    let attributes = AttributeService::new(&repos.attribute);
    for (key, base_type, config) in [
        ("Level", "text", json!({"default": "medium"})),
        (
            "Labels",
            "list",
            json!({"list_type": "text", "default": ["a", "b"]}),
        ),
        ("Colour", "text", json!({})),
    ] {
        attributes
            .add_attribute_kind(
                &AddAttributeKindDto {
                    key: key.into(),
                    description: String::new(),
                    base_type: base_type.into(),
                    config,
                },
                &id(1),
            )
            .unwrap()
            .unwrap();
    }
    f.set_text(1, "Level", "high").await;

    // A kind without a default blocks, even alongside kinds that have one.
    let blocked = f
        .types
        .update_item_type(&f.require(&["Priority", "Colour"], false), &id(1));
    assert!(
        matches!(blocked, Err(ItemTypeServiceError::WouldInvalidate { .. })),
        "{blocked:?}"
    );
    assert!(f.required().is_empty());
    assert_eq!(
        f.count("SELECT COUNT(*) FROM attribute WHERE key = 'Priority'")
            .await,
        0
    );

    let change = f
        .types
        .update_item_type(&f.require(&["Priority", "Level", "Labels"], false), &id(1))
        .unwrap()
        .unwrap();

    assert_eq!(change.report.backfilled["Priority"], 3);
    assert_eq!(
        change.report.backfilled["Level"], 2,
        "item 1 already had a value"
    );
    assert_eq!(change.report.backfilled["Labels"], 3);
    assert_eq!(
        f.count("SELECT COUNT(*) FROM attribute WHERE key = 'Priority' AND value_int = 3")
            .await,
        3
    );
    assert_eq!(
        f.count("SELECT COUNT(*) FROM attribute WHERE key = 'Level' AND value_text = 'high' AND item_id = 1").await,
        1
    );
    assert_eq!(
        f.count("SELECT COUNT(*) FROM attribute_list_value WHERE key = 'Labels'")
            .await,
        6
    );
    assert!(
        f.types
            .list_non_compliant(&f.type_id, 50, 0, &id(1))
            .unwrap()
            .items
            .is_empty()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn backfill_is_atomic_when_a_later_write_fails() {
    let defaults = MapDefaults(HashMap::from([
        ("Rating".to_string(), text("3")),
        ("Note".to_string(), text("n")),
    ]));
    let f = fixture(defaults).await;
    sqlx::query(
        "CREATE TRIGGER fail_note BEFORE INSERT ON attribute WHEN NEW.key = 'Note'
         BEGIN SELECT RAISE(ABORT, 'boom'); END",
    )
    .execute(&f.pool)
    .await
    .unwrap();

    let result = f
        .types
        .update_item_type(&f.require(&["Rating", "Note"], false), &id(1));

    assert!(
        matches!(result, Err(ItemTypeServiceError::Repo(_))),
        "{result:?}"
    );
    assert!(f.required().is_empty());
    assert_eq!(
        f.count("SELECT COUNT(*) FROM attribute WHERE key = 'Rating'")
            .await,
        0,
        "first backfill rolled back"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn adding_through_attr_kind_route_checks_and_dry_runs_too() {
    let f = fixture(MapDefaults::default()).await;

    let dry = f
        .types
        .add_attr_kinds_to_item_type(&vec!["Rating".into()], &f.type_id, &id(1), true)
        .unwrap();
    assert_eq!(dry.non_compliant.total, 3);
    assert!(f.required().is_empty());

    let err =
        f.types
            .add_attr_kinds_to_item_type(&vec!["Rating".into()], &f.type_id, &id(1), false);
    assert!(
        matches!(err, Err(ItemTypeServiceError::WouldInvalidate { .. })),
        "{err:?}"
    );
    assert!(f.required().is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn required_relationship_has_no_default_and_must_already_hold() {
    // An item default is ignored: relationships never backfill.
    let defaults = MapDefaults(HashMap::from([(
        "Parent".to_string(),
        Attribute::List(vec![AttributeScalar::Item(id(4))]),
    )]));
    let f = fixture(defaults).await;

    let dry = f
        .types
        .update_item_type(&f.require(&["Parent"], true), &id(1))
        .unwrap()
        .unwrap();
    assert_eq!(dry.report.non_compliant.by_key["Parent"], 3);
    assert!(
        f.types
            .update_item_type(&f.require(&["Parent"], false), &id(1))
            .is_err()
    );

    for item in 1..=3 {
        sqlx::query("INSERT INTO attribute_list_value (item_id, key, ordinal, value_item_id) VALUES (?, 'Parent', 0, 4)")
            .bind(item)
            .execute(&f.pool)
            .await
            .unwrap();
    }
    f.types
        .update_item_type(&f.require(&["Parent"], false), &id(1))
        .unwrap();
    assert_eq!(f.required(), ["Parent"]);
}

#[tokio::test(flavor = "multi_thread")]
async fn removing_a_requirement_never_validates() {
    let f = fixture(MapDefaults::default()).await;
    f.force_required("Rating").await;

    let change = f
        .types
        .update_item_type(&f.require(&[], false), &id(1))
        .unwrap()
        .unwrap();

    assert_eq!(change.report.removed, ["Rating"]);
    assert!(f.required().is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn rename_in_the_same_call_commits_with_the_requirement() {
    let f = fixture(MapDefaults::default()).await;
    for item in 1..=3 {
        f.set_text(item, "Rating", "ok").await;
    }
    let mut dto = f.require(&["Rating"], false);
    dto.name = Some("Gadget".into());

    let change = f.types.update_item_type(&dto, &id(1)).unwrap().unwrap();

    assert_eq!(change.item_type.name, "Gadget");
    assert_eq!(f.required(), ["Rating"]);
}

#[tokio::test(flavor = "multi_thread")]
async fn non_compliant_listing_names_missing_keys_pages_and_flags_archived() {
    let f = fixture(MapDefaults::default()).await;
    f.force_required("Rating").await;
    f.force_required("Note").await;
    f.set_text(1, "Rating", "ok").await;
    f.set_text(2, "Rating", "ok").await;
    f.set_text(2, "Note", "n").await;

    let page = f
        .types
        .list_non_compliant(&f.type_id, 50, 0, &id(1))
        .unwrap();

    assert_eq!(page.total, 2);
    let ids: Vec<_> = page.items.iter().map(|i| i.item_id).collect();
    assert_eq!(ids, [1, 3]);
    assert_eq!(page.items[0].missing, ["Note"]);
    assert_eq!(page.items[1].missing, ["Note", "Rating"]);
    assert!(!page.items[0].archived && page.items[1].archived);

    let second = f
        .types
        .list_non_compliant(&f.type_id, 1, 1, &id(1))
        .unwrap();
    assert_eq!(
        (second.total, second.items.len(), second.items[0].item_id),
        (2, 1, 3)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn items_out_of_compliance_accept_edits_that_add_no_new_gap() {
    let f = fixture(MapDefaults::default()).await;
    f.force_required("Rating").await;
    f.force_required("Note").await;
    f.set_text(1, "Rating", "ok").await;
    let account = account();

    // Unrelated attribute and setting one of two missing keys both pass.
    f.items
        .set_attributes(
            &id(2),
            &HashMap::from([("Tags".to_string(), json!(["x"]))]),
            &account,
        )
        .unwrap();
    f.items
        .set_attributes(
            &id(2),
            &HashMap::from([("Rating".to_string(), json!("5"))]),
            &account,
        )
        .unwrap();
    // Removing a key that is currently present adds a gap.
    let removed = f.items.delete_attribute(&id(2), "Rating", &account);
    assert!(
        matches!(&removed, Err(ItemServiceError::InvalidFilter(m)) if m.contains("Rating")),
        "{removed:?}"
    );
    // Removing one already missing is fine; so is clearing an unrelated key.
    f.items.delete_attribute(&id(2), "Note", &account).unwrap();
    f.items.delete_attribute(&id(2), "Tags", &account).unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn create_and_assign_type_stay_strict() {
    let f = fixture(MapDefaults::default()).await;
    f.force_required("Rating").await;
    f.force_required("Note").await;
    let account = account();

    let created = f.items.add_item(
        &AddItemDto {
            title: "New".into(),
            content: String::new(),
            attributes: Some(HashMap::from([("Rating".to_string(), json!("1"))])),
            types: Some(vec!["Thing".into()]),
            links: None,
        },
        &account,
    );
    assert!(
        matches!(&created, Err(ItemServiceError::InvalidFilter(m)) if m.contains("Note") && !m.contains("Rating")),
        "{created:?}"
    );
    let assigned = f.items.assign_type("Thing", &id(4), &account);
    assert!(
        matches!(assigned, Err(ItemServiceError::InvalidFilter(_))),
        "{assigned:?}"
    );
}

/// The PostgreSQL repository runs its own copy of the SQL, so cover reject, dry run, backfill (scalar
/// and list), and the listing there too. Needs Docker:
/// `cargo test -p zealot-infra required_change -- --ignored`.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "starts a PostgreSQL container; requires a running Docker daemon"]
async fn requirement_changes_on_postgres() {
    use sqlx::postgres::PgPoolOptions;
    use testcontainers_modules::{
        postgres::Postgres,
        testcontainers::{ImageExt, runners::AsyncRunner},
    };

    use super::postgres::get_postgres_repos;

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
    for statement in [
        "INSERT INTO account (account_id, username, email, password) VALUES (1, 'u', 'u@example.com', 'x')",
        "INSERT INTO item (item_id, title, content, account_id) VALUES (1, 'One', '', 1), (2, 'Two', '', 1), (3, 'Three', '', 1)",
        "INSERT INTO attribute (item_id, key, value_int) VALUES (3, 'Archived', 1)",
    ] {
        sqlx::query(statement).execute(&pool).await.unwrap();
    }
    let repos = get_postgres_repos(pool.clone());
    let attributes = AttributeService::new(&repos.attribute);
    for (key, base_type, config) in [
        ("Rating", "text", json!({})),
        (
            "Tags",
            "list",
            json!({"list_type": "text", "default": ["a", "b"]}),
        ),
    ] {
        attributes
            .add_attribute_kind(
                &AddAttributeKindDto {
                    key: key.into(),
                    description: String::new(),
                    base_type: base_type.into(),
                    config,
                },
                &id(1),
            )
            .unwrap()
            .unwrap();
    }
    let types = ItemTypeService::new(&repos.item_type)
        .with_defaults(Arc::new(KindDefaults::new(&repos.attribute)));
    let thing = types
        .add_item_type(
            &AddItemTypeDto {
                name: "Thing".into(),
                description: String::new(),
                required_attributes: vec![],
                required_defaults: None,
            },
            &id(1),
        )
        .unwrap()
        .unwrap();
    let type_id = i64::from(thing.type_id);
    for item in 1..=3i64 {
        sqlx::query("INSERT INTO item_item_type_link (item_id, type_id) VALUES ($1, $2)")
            .bind(item as i32)
            .bind(type_id as i32)
            .execute(&pool)
            .await
            .unwrap();
    }
    sqlx::query("INSERT INTO attribute (item_id, key, value_text) VALUES (1, 'Rating', 'ok')")
        .execute(&pool)
        .await
        .unwrap();
    let dto = |keys: &[&str], dry_run| UpdateItemTypeDto {
        type_id,
        name: None,
        description: None,
        required_attributes: Some(keys.iter().map(|k| k.to_string()).collect()),
        dry_run: Some(dry_run),
        required_defaults: None,
    };

    let dry = types
        .update_item_type(&dto(&["Rating"], true), &id(1))
        .unwrap()
        .unwrap();
    assert_eq!(dry.report.non_compliant.total, 2);
    assert_eq!(dry.report.non_compliant.sample_ids["Rating"], [2, 3]);
    let rejected = types.update_item_type(&dto(&["Rating"], false), &id(1));
    assert!(
        matches!(rejected, Err(ItemTypeServiceError::WouldInvalidate { .. })),
        "{rejected:?}"
    );
    assert!(
        types
            .get_item_type(&thing.type_id, &id(1))
            .unwrap()
            .unwrap()
            .required_attributes
            .is_empty()
    );

    let change = types
        .update_item_type(&dto(&["Tags"], false), &id(1))
        .unwrap()
        .unwrap();
    assert_eq!(change.report.backfilled["Tags"], 3);
    let tags: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM attribute_list_value WHERE key = 'Tags'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(tags, 6);

    sqlx::query(
        "INSERT INTO item_type_attribute_kind_link (attribute_kind_id, item_type_id)
         SELECT kind_id, $1 FROM attribute_kind WHERE key = 'Rating'",
    )
    .bind(type_id as i32)
    .execute(&pool)
    .await
    .unwrap();
    let page = types
        .list_non_compliant(&thing.type_id, 50, 0, &id(1))
        .unwrap();
    assert_eq!(page.total, 2);
    assert_eq!(page.items[0].missing, ["Rating"]);
    assert!(page.items[1].archived);
}

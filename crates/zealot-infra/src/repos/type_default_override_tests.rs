//! Type-level default overrides through the services and the real repositories: storing,
//! updating and clearing them, validation against the kind, and the per-member backfill rule
//! (agreeing type defaults win, else the kind default). No production data.
use std::{collections::BTreeMap, sync::Arc};

use serde_json::{Value, json};
use sqlx::{PgPool, SqlitePool, postgres::PgPoolOptions, sqlite::SqlitePoolOptions};
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{ImageExt, runners::AsyncRunner},
};
use zealot_app::{
    repos::ZealotRepos,
    services::{
        attribute::AttributeService,
        item_type::{ItemTypeService, ItemTypeServiceError, KindDefaults},
    },
};
use zealot_domain::{
    attribute::AddAttributeKindDto,
    common::id::Id,
    item_type::{AddItemTypeDto, ItemType, UpdateItemTypeDto},
};

use super::{postgres::get_postgres_repos, sqlite::get_sqlite_repos};

const SEED: &[&str] = &[
    "INSERT INTO account (account_id, username, email, password) VALUES (1, 'u', 'u@example.com', 'x')",
    "INSERT INTO item (item_id, title, content, account_id) VALUES (1, 'One', '', 1), (2, 'Two', '', 1), (3, 'Three', '', 1), (4, 'Four', '', 1)",
];

fn id(n: i64) -> Id {
    Id::try_from(n).unwrap()
}

trait RawDb {
    async fn exec(&self, sql: &str);
    async fn scalar(&self, sql: &str) -> i64;
}
impl RawDb for SqlitePool {
    async fn exec(&self, sql: &str) {
        sqlx::query(sql).execute(self).await.unwrap();
    }
    async fn scalar(&self, sql: &str) -> i64 {
        sqlx::query_scalar(sql).fetch_one(self).await.unwrap()
    }
}
impl RawDb for PgPool {
    async fn exec(&self, sql: &str) {
        sqlx::query(sql).execute(self).await.unwrap();
    }
    async fn scalar(&self, sql: &str) -> i64 {
        sqlx::query_scalar(sql).fetch_one(self).await.unwrap()
    }
}

struct Fixture {
    types: ItemTypeService,
    epic: ItemType,
    goal: ItemType,
    story: ItemType,
}

fn dto(
    type_id: &Id,
    required: &[&str],
    defaults: &[(&str, Value)],
    dry_run: bool,
) -> UpdateItemTypeDto {
    UpdateItemTypeDto {
        type_id: i64::from(*type_id),
        name: None,
        description: None,
        required_attributes: Some(required.iter().map(|k| k.to_string()).collect()),
        required_defaults: Some(
            defaults
                .iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect(),
        ),
        dry_run: Some(dry_run),
    }
}

/// Types Epic (no requirements yet), Goal (Priority 7) and Story (Priority 5), each Priority
/// override set while the type has no members. Kinds: system `Priority` (kind default 3),
/// user `Effort` (integer 1..10, no kind default).
async fn fixture(repos: &ZealotRepos) -> Fixture {
    let attributes = AttributeService::new(&repos.attribute);
    attributes
        .add_attribute_kind(
            &AddAttributeKindDto {
                key: "Effort".into(),
                description: String::new(),
                base_type: "integer".into(),
                config: json!({"min": 1, "max": 10}),
            },
            &id(1),
        )
        .unwrap()
        .unwrap();
    let types = ItemTypeService::new(&repos.item_type)
        .with_defaults(Arc::new(KindDefaults::new(&repos.attribute)));
    let make = |name: &str, required: &[&str], defaults: &[(&str, Value)]| {
        types
            .add_item_type(
                &AddItemTypeDto {
                    name: name.into(),
                    description: String::new(),
                    required_attributes: required.iter().map(|k| k.to_string()).collect(),
                    required_defaults: if defaults.is_empty() {
                        None
                    } else {
                        Some(
                            defaults
                                .iter()
                                .map(|(k, v)| (k.to_string(), v.clone()))
                                .collect(),
                        )
                    },
                },
                &id(1),
            )
            .unwrap()
            .unwrap()
    };
    let epic = make("Epic", &[], &[]);
    let goal = make("Goal", &["Priority"], &[("Priority", json!(7))]);
    let story = make("Story", &["Priority"], &[("Priority", json!(5))]);
    Fixture {
        types,
        epic,
        goal,
        story,
    }
}

fn required_defaults(types: &ItemTypeService, t: &ItemType) -> BTreeMap<String, Value> {
    types
        .get_item_type(&t.type_id, &id(1))
        .unwrap()
        .unwrap()
        .required_defaults
}

async fn link(db: &impl RawDb, item: i64, t: &ItemType) {
    db.exec(&format!(
        "INSERT INTO item_item_type_link (item_id, type_id) VALUES ({item}, {})",
        i64::from(t.type_id)
    ))
    .await;
}

/// Epic holds items 1-4; item 2 and 4 also have Goal (7); items 3 and 4 also have Story (5).
async fn assign_members(db: &impl RawDb, f: &Fixture) {
    for item in 1..=4 {
        link(db, item, &f.epic).await;
    }
    link(db, 2, &f.goal).await;
    link(db, 4, &f.goal).await;
    link(db, 3, &f.story).await;
    link(db, 4, &f.story).await;
}

async fn check_store_update_clear_and_drop(_db: &impl RawDb, repos: ZealotRepos) {
    let f = fixture(&repos).await;
    assert_eq!(
        required_defaults(&f.types, &f.goal)["Priority"],
        json!(7),
        "set on create"
    );

    let change = f
        .types
        .update_item_type(
            &dto(
                &f.epic.type_id,
                &["Priority"],
                &[("Priority", json!(5))],
                false,
            ),
            &id(1),
        )
        .unwrap()
        .unwrap();
    assert_eq!(change.item_type.required_defaults["Priority"], json!(5));

    f.types
        .update_item_type(
            &dto(
                &f.epic.type_id,
                &["Priority"],
                &[("Priority", json!(8))],
                false,
            ),
            &id(1),
        )
        .unwrap();
    assert_eq!(
        required_defaults(&f.types, &f.epic)["Priority"],
        json!(8),
        "updated"
    );

    // Leaving `required_defaults` out keeps it; null clears it.
    let mut keep = dto(&f.epic.type_id, &["Priority"], &[], false);
    keep.required_defaults = None;
    f.types.update_item_type(&keep, &id(1)).unwrap();
    assert_eq!(required_defaults(&f.types, &f.epic)["Priority"], json!(8));
    f.types
        .update_item_type(
            &dto(
                &f.epic.type_id,
                &["Priority"],
                &[("Priority", Value::Null)],
                false,
            ),
            &id(1),
        )
        .unwrap();
    assert!(required_defaults(&f.types, &f.epic).is_empty(), "cleared");

    // Dropping the requirement drops its default; re-adding starts without one.
    f.types
        .update_item_type(
            &dto(
                &f.epic.type_id,
                &["Priority"],
                &[("Priority", json!(4))],
                false,
            ),
            &id(1),
        )
        .unwrap();
    f.types
        .update_item_type(&dto(&f.epic.type_id, &[], &[], false), &id(1))
        .unwrap();
    f.types
        .update_item_type(&dto(&f.epic.type_id, &["Priority"], &[], false), &id(1))
        .unwrap();
    assert!(required_defaults(&f.types, &f.epic).is_empty());
}

async fn check_validation(_db: &impl RawDb, repos: ZealotRepos) {
    let f = fixture(&repos).await;
    let bad = |defaults: &[(&str, Value)], required: &[&str]| {
        f.types
            .update_item_type(&dto(&f.epic.type_id, required, defaults, false), &id(1))
    };

    for (required, defaults) in [
        (&["Effort"][..], vec![("Effort", json!(99))]),
        (&["Effort"], vec![("Effort", json!("many"))]),
        (&["Status"], vec![("Status", json!("Not a status"))]),
        (&["Parent"], vec![("Parent", json!([1]))]),
        (&["Effort"], vec![("Priority", json!(3))]),
    ] {
        let result = bad(&defaults, required);
        assert!(
            matches!(result, Err(ItemTypeServiceError::InvalidDefault(_))),
            "{required:?} {defaults:?}: {result:?}"
        );
    }
    assert!(required_defaults(&f.types, &f.epic).is_empty());
    assert!(
        f.types
            .get_item_type(&f.epic.type_id, &id(1))
            .unwrap()
            .unwrap()
            .required_attributes
            .is_empty()
    );

    // A new type with an invalid default is not created.
    let created = f.types.add_item_type(
        &AddItemTypeDto {
            name: "Broken".into(),
            description: String::new(),
            required_attributes: vec!["Effort".into()],
            required_defaults: Some(BTreeMap::from([("Effort".to_string(), json!(0))])),
        },
        &id(1),
    );
    assert!(
        matches!(created, Err(ItemTypeServiceError::InvalidDefault(_))),
        "{created:?}"
    );
    assert!(
        f.types
            .get_item_type_by_name("Broken", &id(1))
            .unwrap()
            .is_none()
    );

    // System types are read-only, defaults included.
    let plan = f
        .types
        .get_item_type_by_name("Plan", &id(1))
        .unwrap()
        .unwrap();
    let result = f.types.update_item_type(
        &dto(
            &plan.type_id,
            &["Status"],
            &[("Status", json!("To Do"))],
            false,
        ),
        &id(1),
    );
    assert!(
        matches!(result, Err(ItemTypeServiceError::ReadOnly(_))),
        "{result:?}"
    );
}

async fn priority_of(db: &impl RawDb, item: i64) -> i64 {
    db.scalar(&format!("SELECT CAST(value_int AS BIGINT) FROM attribute WHERE item_id = {item} AND key = 'Priority'"))
        .await
}

async fn check_backfill_applies_the_multi_type_rule(db: &impl RawDb, repos: ZealotRepos) {
    let f = fixture(&repos).await;
    assign_members(db, &f).await;

    // Epic gains Priority with its own default 5. Item 1: only Epic -> 5. Item 2: Epic 5 vs
    // Goal 7 disagree -> kind default 3. Item 3: Epic 5 and Story 5 agree -> 5. Item 4: all
    // three, 5/7/5 disagree -> 3.
    let change = f
        .types
        .update_item_type(
            &dto(
                &f.epic.type_id,
                &["Priority"],
                &[("Priority", json!(5))],
                false,
            ),
            &id(1),
        )
        .unwrap()
        .unwrap();

    assert_eq!(change.report.backfilled["Priority"], 4);
    assert_eq!(change.report.non_compliant.total, 0);
    assert_eq!(priority_of(db, 1).await, 5);
    assert_eq!(priority_of(db, 2).await, 3);
    assert_eq!(priority_of(db, 3).await, 5);
    assert_eq!(priority_of(db, 4).await, 3);
}

async fn check_type_default_alone_and_kind_default_alone(db: &impl RawDb, repos: ZealotRepos) {
    let f = fixture(&repos).await;
    assign_members(db, &f).await;

    // Effort has no kind default; Epic's own default covers every member (Goal/Story do not
    // require Effort, so they do not take part).
    let change = f
        .types
        .update_item_type(
            &dto(&f.epic.type_id, &["Effort"], &[("Effort", json!(2))], false),
            &id(1),
        )
        .unwrap()
        .unwrap();
    assert_eq!(change.report.backfilled["Effort"], 4);
    assert_eq!(
        db.scalar("SELECT COUNT(*) FROM attribute WHERE key = 'Effort' AND value_int = 2")
            .await,
        4
    );

    // Without any type default the kind default (Priority 3) fills everyone.
    let story2 = f
        .types
        .add_item_type(
            &AddItemTypeDto {
                name: "Task2".into(),
                description: String::new(),
                required_attributes: vec![],
                required_defaults: None,
            },
            &id(1),
        )
        .unwrap()
        .unwrap();
    link(db, 1, &story2).await;
    let plain = f
        .types
        .update_item_type(&dto(&story2.type_id, &["Priority"], &[], false), &id(1))
        .unwrap()
        .unwrap();
    assert_eq!(plain.report.backfilled["Priority"], 1);
    assert_eq!(priority_of(db, 1).await, 3);
}

async fn check_conflict_without_kind_default_rejects(db: &impl RawDb, repos: ZealotRepos) {
    let f = fixture(&repos).await;
    // Goal and Story gain Effort with different defaults while empty; then link members.
    f.types
        .update_item_type(
            &dto(
                &f.goal.type_id,
                &["Priority", "Effort"],
                &[("Effort", json!(7))],
                false,
            ),
            &id(1),
        )
        .unwrap();
    f.types
        .update_item_type(
            &dto(
                &f.story.type_id,
                &["Priority", "Effort"],
                &[("Effort", json!(4))],
                false,
            ),
            &id(1),
        )
        .unwrap();
    assign_members(db, &f).await;

    // Epic default 7 agrees with Goal on item 2 but not with Story on item 3; item 4 has all
    // three (7/7/4) and conflicts; no kind default exists, so items 3 and 4 have none.
    let attempt = dto(&f.epic.type_id, &["Effort"], &[("Effort", json!(7))], false);
    let dry = f
        .types
        .update_item_type(
            &dto(&f.epic.type_id, &["Effort"], &[("Effort", json!(7))], true),
            &id(1),
        )
        .unwrap()
        .unwrap();
    assert_eq!(dry.report.non_compliant.total, 2);
    assert_eq!(dry.report.non_compliant.by_key["Effort"], 2);
    assert_eq!(dry.report.non_compliant.sample_ids["Effort"], [3, 4]);
    assert_eq!(
        dry.report.backfilled["Effort"], 2,
        "items 1 and 2 would be filled"
    );

    let result = f.types.update_item_type(&attempt, &id(1));
    assert!(
        matches!(result, Err(ItemTypeServiceError::WouldInvalidate { .. })),
        "{result:?}"
    );
    assert_eq!(
        db.scalar("SELECT COUNT(*) FROM attribute WHERE key = 'Effort'")
            .await,
        0,
        "nothing written"
    );
    assert!(required_defaults(&f.types, &f.epic).is_empty());
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
        pool.exec(statement).await;
    }
    pool
}

macro_rules! sqlite_test {
    ($name:ident, $check:ident) => {
        #[tokio::test(flavor = "multi_thread")]
        async fn $name() {
            let pool = sqlite_pool().await;
            $check(&pool, get_sqlite_repos(pool.clone())).await;
        }
    };
}

sqlite_test!(
    store_update_clear_and_drop_with_requirement_on_sqlite,
    check_store_update_clear_and_drop
);
sqlite_test!(
    invalid_type_defaults_are_rejected_on_sqlite,
    check_validation
);
sqlite_test!(
    backfill_applies_the_multi_type_rule_on_sqlite,
    check_backfill_applies_the_multi_type_rule
);
sqlite_test!(
    type_default_alone_and_kind_default_alone_on_sqlite,
    check_type_default_alone_and_kind_default_alone
);
sqlite_test!(
    conflict_without_kind_default_rejects_on_sqlite,
    check_conflict_without_kind_default_rejects
);

/// One container runs every check, each on a fresh schema.
/// `cargo test -p zealot-infra type_default_override -- --ignored` (needs Docker).
#[tokio::test(flavor = "multi_thread")]
#[ignore = "starts a PostgreSQL container; requires a running Docker daemon"]
async fn type_default_overrides_on_postgres() {
    let container = Postgres::default()
        .with_tag("16-alpine")
        .start()
        .await
        .unwrap();
    let port = container.get_host_port_ipv4(5432).await.unwrap();
    let admin = PgPoolOptions::new()
        .max_connections(1)
        .connect(&format!(
            "postgres://postgres:postgres@127.0.0.1:{port}/postgres"
        ))
        .await
        .unwrap();

    for (n, name) in ["store", "validate", "multi", "alone", "conflict"]
        .into_iter()
        .enumerate()
    {
        admin.exec(&format!("CREATE DATABASE t{n}_{name}")).await;
        let pool = PgPoolOptions::new()
            .max_connections(2)
            .connect(&format!(
                "postgres://postgres:postgres@127.0.0.1:{port}/t{n}_{name}"
            ))
            .await
            .unwrap();
        sqlx::migrate!("migrations/postgres")
            .run(&pool)
            .await
            .unwrap();
        for statement in SEED {
            pool.exec(statement).await;
        }
        let repos = get_postgres_repos(pool.clone());
        match name {
            "store" => check_store_update_clear_and_drop(&pool, repos).await,
            "validate" => check_validation(&pool, repos).await,
            "multi" => check_backfill_applies_the_multi_type_rule(&pool, repos).await,
            "alone" => check_type_default_alone_and_kind_default_alone(&pool, repos).await,
            _ => check_conflict_without_kind_default_rejects(&pool, repos).await,
        }
    }
}

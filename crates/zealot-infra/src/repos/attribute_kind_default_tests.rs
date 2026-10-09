//! Attribute kind defaults: system defaults from the migrations, and validated add/update/clear
//! through `AttributeService` on the real repositories. No production data.
//!
//! The PostgreSQL test starts a throwaway container, so it is `#[ignore]`d and needs Docker:
//! `cargo test -p zealot-infra attribute_kind_default -- --ignored`.
use serde_json::{Value, json};
use sqlx::{PgPool, postgres::PgPoolOptions, sqlite::SqlitePoolOptions};
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{ImageExt, runners::AsyncRunner},
};
use zealot_app::services::attribute::{AttributeService, AttributeServiceError};
use zealot_domain::{
    attribute::{AddAttributeKindDto, Attribute, AttributeScalar, UpdateAttributeKindDto},
    common::id::Id,
};

use super::{ZealotRepos, postgres::get_postgres_repos, sqlite::get_sqlite_repos};

const ACCOUNT: &str = "INSERT INTO account (account_id, username, email, password) VALUES (1, 'u', 'u@example.com', 'x')";

fn account_id() -> Id {
    Id::try_from(1i64).unwrap()
}

fn add(key: &str, base_type: &str, config: Value) -> AddAttributeKindDto {
    AddAttributeKindDto {
        key: key.into(),
        description: format!("{key} test kind"),
        base_type: base_type.into(),
        config,
    }
}

fn int(value: i64) -> Option<Attribute> {
    Some(Attribute::Scalar(AttributeScalar::Integer(value)))
}

fn text(value: &str) -> Option<Attribute> {
    Some(Attribute::Scalar(AttributeScalar::Dropdown(value.into())))
}

fn check_defaults(repos: ZealotRepos) {
    let svc = AttributeService::new(&repos.attribute);
    let me = account_id();

    // System kinds carry their defaults from the migration, and still validate.
    let status = svc.get_kind_by_key("Status", &me).unwrap().unwrap();
    assert_eq!(status.default, text("To Do"));
    let priority = svc.get_kind_by_key("Priority", &me).unwrap().unwrap();
    assert_eq!(priority.default, int(3));
    assert!(
        svc.get_kind_by_key("Rating", &me)
            .unwrap()
            .is_none_or(|k| k.default.is_none())
    );

    // A user kind stores a valid default and reads it back.
    let config = json!({"min": 1, "max": 5, "default": 2});
    let rating = svc
        .add_attribute_kind(&add("Effort", "integer", config), &me)
        .unwrap()
        .unwrap();
    assert_eq!(rating.default, int(2));
    assert_eq!(
        svc.get_kind_by_key("Effort", &me).unwrap().unwrap().default,
        int(2)
    );

    // Invalid defaults are rejected on add and nothing is stored.
    for bad in [json!(9), json!("two")] {
        let config = json!({"min": 1, "max": 5, "default": bad});
        let result = svc.add_attribute_kind(&add("Bad", "integer", config), &me);
        assert!(
            matches!(result, Err(AttributeServiceError::InvalidDefault(_))),
            "{result:?}"
        );
    }
    assert!(svc.get_kind_by_key("Bad", &me).unwrap().is_none());

    // Update replaces the config: a new valid default is stored, an invalid one is refused,
    // and a config without `default` clears it.
    let update = |config: Value| UpdateAttributeKindDto {
        kind_id: i64::from(rating.kind_id),
        key: None,
        description: None,
        base_type: None,
        config: Some(config),
    };
    let updated = svc
        .update_attribute_kind(&update(json!({"min": 1, "max": 5, "default": 4})), &me)
        .unwrap()
        .unwrap();
    assert_eq!(updated.default, int(4));
    let result = svc.update_attribute_kind(&update(json!({"min": 1, "max": 5, "default": 6})), &me);
    assert!(
        matches!(result, Err(AttributeServiceError::InvalidDefault(_))),
        "{result:?}"
    );
    assert_eq!(
        svc.get_kind_by_key("Effort", &me).unwrap().unwrap().default,
        int(4)
    );
    let cleared = svc
        .update_attribute_kind(&update(json!({"min": 1, "max": 5})), &me)
        .unwrap()
        .unwrap();
    assert_eq!(cleared.default, None);

    // Dropdown and list-of-dropdown defaults follow the dropdown's values.
    let values = json!(["a", "b"]);
    let dropdown = add(
        "Mode",
        "dropdown",
        json!({"values": values, "default": "b"}),
    );
    assert_eq!(
        svc.add_attribute_kind(&dropdown, &me)
            .unwrap()
            .unwrap()
            .default,
        text("b")
    );
    let bad = add(
        "Mode2",
        "dropdown",
        json!({"values": ["a", "b"], "default": "z"}),
    );
    assert!(matches!(
        svc.add_attribute_kind(&bad, &me),
        Err(AttributeServiceError::InvalidDefault(_))
    ));
    let tags = json!({"list_type": "dropdown", "values": ["a", "b"], "default": ["a"]});
    let tags = svc
        .add_attribute_kind(&add("Tags2", "list", tags), &me)
        .unwrap()
        .unwrap();
    assert_eq!(
        tags.default,
        Some(Attribute::List(vec![AttributeScalar::Dropdown("a".into())]))
    );
    let bad = json!({"list_type": "dropdown", "values": ["a", "b"], "default": ["z"]});
    assert!(matches!(
        svc.add_attribute_kind(&add("Tags3", "list", bad), &me),
        Err(AttributeServiceError::InvalidDefault(_))
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn kind_defaults_on_sqlite() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("migrations/sqlite")
        .run(&pool)
        .await
        .unwrap();
    sqlx::query(ACCOUNT).execute(&pool).await.unwrap();
    check_defaults(get_sqlite_repos(pool));
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "starts a PostgreSQL container; requires a running Docker daemon"]
async fn kind_defaults_on_postgres() {
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
    sqlx::query(ACCOUNT).execute(&pool).await.unwrap();
    check_defaults(get_postgres_repos(pool));
}

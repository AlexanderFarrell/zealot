//! Email, telephone, url, media and place attribute kinds, and multi-select, through the
//! service and the real sqlite repositories. No production data.
use super::sqlite::get_sqlite_repos;
use serde_json::{Value, json};
use sqlx::sqlite::SqlitePoolOptions;
use std::{collections::HashMap, sync::Arc};
use zealot_app::{
    ports::events::{EventPort, NoopEventPort},
    services::item::ItemService,
};
use zealot_domain::{
    account::Account,
    attribute::AddAttributeKindDto,
    common::{email::Email, id::Id},
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

fn kind(key: &str, base_type: &str, config: Value) -> AddAttributeKindDto {
    AddAttributeKindDto {
        key: key.into(),
        description: format!("{key} test kind"),
        base_type: base_type.into(),
        config,
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn semantic_kinds_validate_and_round_trip() {
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
    sqlx::query("INSERT INTO item (item_id, title, content, account_id) VALUES (1, 'Contact', '', 1)")
        .execute(&pool)
        .await
        .unwrap();
    let repos = get_sqlite_repos(pool.clone());
    let user = account();

    // The migration keeps the links from item types to their attribute kinds.
    let links: i64 = sqlx::query_scalar("SELECT count(*) FROM item_type_attribute_kind_link")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(links > 0, "migration 0017 must preserve item type links");

    for (key, base) in [
        ("Contact Email", "email"),
        ("Contact Phone", "telephone"),
        ("Homepage", "url"),
        ("Photo", "media"),
        ("Home", "place"),
    ] {
        repos
            .attribute
            .add_attribute_kind(&kind(key, base, json!({})), &user.account_id)
            .unwrap()
            .unwrap_or_else(|| panic!("{base} kind created"));
    }
    repos
        .attribute
        .add_attribute_kind(
            &kind(
                "Tags",
                "list",
                json!({"list_type": "dropdown", "values": ["a", "b"]}),
            ),
            &user.account_id,
        )
        .unwrap();

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

    let good: HashMap<String, Value> = HashMap::from([
        ("Contact Email".into(), json!("me@example.com")),
        ("Contact Phone".into(), json!("+1 (555) 123-4567")),
        ("Homepage".into(), json!("https://example.com/a?b=1")),
        ("Photo".into(), json!("/media/cat.png")),
        ("Home".into(), json!("51.5, -0.12")),
    ]);
    svc.set_attributes(&id(1), &good, &user).unwrap();
    let item = svc.get_item_by_id(&id(1), &user).unwrap().unwrap();
    let attributes = serde_json::to_value(&item.attributes).unwrap();
    assert_eq!(attributes["Contact Email"], "me@example.com");
    assert_eq!(attributes["Homepage"], "https://example.com/a?b=1");
    assert_eq!(attributes["Photo"], "/media/cat.png");
    assert_eq!(attributes["Home"], "51.5, -0.12");

    for (key, bad) in [
        ("Contact Email", json!("not-an-email")),
        ("Contact Phone", json!("call me")),
        ("Homepage", json!("javascript:alert(1)")),
        ("Photo", json!("cat.png")),
        ("Home", json!("")),
    ] {
        let err = svc
            .set_attributes(&id(1), &HashMap::from([(key.to_string(), bad)]), &user)
            .unwrap_err()
            .to_string();
        assert!(err.contains(key), "{err}");
    }

    svc.set_attributes(
        &id(1),
        &HashMap::from([("Tags".to_string(), json!(["a", "b"]))]),
        &user,
    )
    .unwrap();
    assert!(
        svc.set_attributes(
            &id(1),
            &HashMap::from([("Tags".to_string(), json!(["a", "z"]))]),
            &user
        )
        .is_err()
    );
}

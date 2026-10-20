//! Tools reject parameters they do not define instead of ignoring them.
//!
//! Silently dropping a misplaced field (such as `attributes` on `update_item`)
//! made calls report success while writing nothing, so every parameter struct
//! denies unknown fields and the registered schemas say so.

use rmcp::{handler::server::wrapper::Parameters, model::ErrorCode};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use zealot_mcp::{
    client::ZealotClient,
    tools::{
        NoParams, ZealotServer,
        wiki::{AssignTypeParams, UpdateAttributesParams, UpdateItemParams},
    },
};

/// Error text from deserializing `args` into `T`, panicking if it is accepted.
fn rejection<T: DeserializeOwned>(args: Value) -> String {
    match serde_json::from_value::<T>(args) {
        Ok(_) => panic!("unknown field was accepted"),
        Err(e) => e.to_string(),
    }
}

#[test]
fn update_item_attributes_rejects_attributes_field() {
    let msg =
        rejection::<UpdateAttributesParams>(json!({"item": "3633", "attributes": {"Priority": 4}}));
    assert!(msg.contains("unknown field `attributes`"), "{msg}");
    for accepted in ["`item`", "`set`", "`remove`", "`rename`"] {
        assert!(msg.contains(accepted), "{msg} should list {accepted}");
    }
}

#[test]
fn update_item_attributes_accepts_set() {
    let p: UpdateAttributesParams =
        serde_json::from_value(json!({"item": "3633", "set": {"Priority": 4}})).unwrap();
    assert_eq!(p.set.unwrap()["Priority"], json!(4));
}

#[test]
fn update_item_rejects_attributes_parent_and_types() {
    for field in ["attributes", "parent", "types"] {
        let msg = rejection::<UpdateItemParams>(json!({"item": "1", "title": "t", field: {}}));
        assert!(msg.contains(&format!("unknown field `{field}`")), "{msg}");
        assert!(
            msg.contains("`title`") && msg.contains("`content`"),
            "{msg}"
        );
    }
}

#[test]
fn assign_item_type_rejects_extra_attributes() {
    let msg = rejection::<AssignTypeParams>(
        json!({"item": "1", "type_name": "Goal", "attributes": {"Status": "To Do"}}),
    );
    assert!(msg.contains("unknown field `attributes`"), "{msg}");
}

#[test]
fn tools_without_parameters_reject_arguments() {
    let msg = rejection::<NoParams>(json!({"dry_run": true}));
    assert!(msg.contains("unknown field `dry_run`"), "{msg}");
    serde_json::from_value::<NoParams>(json!({})).unwrap();
}

/// Every object schema that lists properties must close the object, so each
/// tool's parameters (and nested parameter structs) reject unknown fields.
/// Free-form payloads declare no properties and are exempt: the rule trigger
/// and attribute kind `config` objects (`JsonObject`), and the values of
/// `set`, `attributes` and filter `value`, which are arbitrary JSON by design.
#[test]
fn every_tool_schema_closes_its_objects() {
    fn open_objects(value: &Value, at: &str, open: &mut Vec<String>, checked: &mut usize) {
        match value {
            Value::Object(map) => {
                if map.contains_key("properties") {
                    *checked += 1;
                }
                if map.contains_key("properties")
                    && map.get("additionalProperties") != Some(&json!(false))
                {
                    open.push(at.to_string());
                }
                for (k, v) in map {
                    open_objects(v, &format!("{at}/{k}"), open, checked);
                }
            }
            Value::Array(values) => {
                for (i, v) in values.iter().enumerate() {
                    open_objects(v, &format!("{at}/{i}"), open, checked);
                }
            }
            _ => {}
        }
    }
    let mut open = Vec::new();
    let mut checked = 0;
    for tool in ZealotServer::tool_router().list_all() {
        let schema = Value::Object((*tool.input_schema).clone());
        open_objects(&schema, &tool.name, &mut open, &mut checked);
    }
    assert!(
        open.is_empty(),
        "schemas that accept unknown fields: {open:?}"
    );
}

#[tokio::test]
async fn update_item_attributes_with_nothing_to_change_is_an_error() {
    let mock = MockServer::start().await;
    let server = ZealotServer {
        client: ZealotClient::new(mock.uri(), "key"),
        journal_item: None,
    };
    for p in [
        json!({"item": "7"}),
        json!({"item": "7", "set": {}, "rename": [], "remove": []}),
    ] {
        let err = server
            .update_item_attributes(Parameters(serde_json::from_value(p).unwrap()))
            .await
            .expect_err("a call that changes nothing must fail");
        assert_eq!(err.code, ErrorCode::INVALID_PARAMS);
        assert!(err.message.contains("nothing to change"), "{}", err.message);
    }
    assert!(mock.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn update_item_attributes_reply_lists_changed_keys() {
    let mock = MockServer::start().await;
    let item = json!({
        "item_id": 7, "title": "Seven", "content": "", "attributes": {}, "types": [], "links": []
    });
    Mock::given(method("GET"))
        .and(path("/item/id/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&item))
        .mount(&mock)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/item/7/attr"))
        .and(body_json(json!({"Priority": 4, "Status": "Working"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(&item))
        .mount(&mock)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/item/7/attr/Old"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&item))
        .mount(&mock)
        .await;
    let server = ZealotServer {
        client: ZealotClient::new(mock.uri(), "key"),
        journal_item: None,
    };
    let params = serde_json::from_value(
        json!({"item": "7", "set": {"Status": "Working", "Priority": 4}, "remove": ["Old"]}),
    )
    .unwrap();
    let result = server
        .update_item_attributes(Parameters(params))
        .await
        .unwrap();
    let reply: Value = serde_json::from_str(&result.content[0].as_text().unwrap().text).unwrap();
    assert_eq!(reply["item_id"], 7);
    assert_eq!(reply["set"], json!(["Priority", "Status"]));
    assert_eq!(reply["removed"], json!(["Old"]));
    assert_eq!(reply["renamed"], json!([]));
}

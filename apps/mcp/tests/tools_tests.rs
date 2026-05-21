use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::ErrorCode;
use serde_json::json;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use zealot_mcp::client::ZealotClient;
use zealot_mcp::tools::ZealotServer;
use zealot_mcp::tools::automation::{
    AttributeKeyParam, CreateAttributeKindParams, CreateItemTypeParams,
    CreateRuleParams, ItemTypeIdParam, ItemTypeNameParam, JsonObject, RuleIdParam,
    UpdateAttributeKindParams, UpdateItemTypeParams, UpdateRuleParams,
};
use zealot_mcp::tools::media::{CreateFolderParams, DeleteMediaParams, MediaPathParam};
use zealot_mcp::tools::planner::{
    AddCommentParams, CommentIdParam, DateParam, ItemIdParam as PlannerItemIdParam,
    MonthYearParam, UpdateCommentParams, UpdateRepeatParams, WeekParam,
};
use zealot_mcp::tools::wiki::{
    AssignTypeParams, CreateItemParams, DeleteAttributeParams, GetItemByTitleParams,
    ItemIdParam, ListItemsParams, RecentItemsParams, SearchItemsParams, SetAttributesParams,
    UpdateItemParams,
};

async fn make_server() -> (ZealotServer, MockServer) {
    let mock = MockServer::start().await;
    let server = ZealotServer {
        client: ZealotClient::new(mock.uri(), "key"),
    };
    (server, mock)
}

fn text_of(result: rmcp::model::CallToolResult) -> String {
    result.content[0].as_text().unwrap().text.clone()
}

// ── api_err() ─────────────────────────────────────────────────────────────────

#[test]
fn api_err_not_found_is_invalid_params() {
    use zealot_mcp::client::ApiError;
    use zealot_mcp::tools::api_err;

    let err = api_err(ApiError::NotFound);
    assert_eq!(err.code, ErrorCode::INVALID_PARAMS);
    assert!(err.message.contains("not found"));
}

#[test]
fn api_err_http_error_is_internal_error() {
    use reqwest::StatusCode;
    use zealot_mcp::client::ApiError;
    use zealot_mcp::tools::api_err;

    let err = api_err(ApiError::Http {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        message: "boom".to_string(),
    });
    assert_eq!(err.code, ErrorCode::INTERNAL_ERROR);
    assert!(err.message.contains("boom"));
}

#[tokio::test]
async fn api_err_network_error_is_internal_error() {
    // Port 1 is always refused — triggers ApiError::Request
    let server = ZealotServer {
        client: ZealotClient::new("http://127.0.0.1:1", "key"),
    };
    let result = server
        .get_item(Parameters(ItemIdParam { id: 1 }))
        .await;
    match result {
        Err(e) => assert_eq!(e.code, ErrorCode::INTERNAL_ERROR),
        Ok(_) => panic!("expected error"),
    }
}

// ── Wiki: list_items ──────────────────────────────────────────────────────────

#[tokio::test]
async fn list_items_no_filter() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/item"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .list_items(Parameters(ListItemsParams { type_filter: None }))
        .await
        .unwrap();
}

#[tokio::test]
async fn list_items_with_filter_url_encoded() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/item"))
        .and(query_param("type", "My Type"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .list_items(Parameters(ListItemsParams {
            type_filter: Some("My Type".to_string()),
        }))
        .await
        .unwrap();
}

// ── Wiki: list_recent_items ───────────────────────────────────────────────────

#[tokio::test]
async fn list_recent_items_defaults() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/item/recent"))
        .and(query_param("limit", "30"))
        .and(query_param("offset", "0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .list_recent_items(Parameters(RecentItemsParams {
            limit: None,
            offset: None,
        }))
        .await
        .unwrap();
}

#[tokio::test]
async fn list_recent_items_custom_pagination() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/item/recent"))
        .and(query_param("limit", "10"))
        .and(query_param("offset", "5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .list_recent_items(Parameters(RecentItemsParams {
            limit: Some(10),
            offset: Some(5),
        }))
        .await
        .unwrap();
}

// ── Wiki: search_items ────────────────────────────────────────────────────────

#[tokio::test]
async fn search_items_url_encodes_spaces() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/item/search"))
        .and(query_param("term", "hello world"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .search_items(Parameters(SearchItemsParams {
            term: "hello world".to_string(),
        }))
        .await
        .unwrap();
}

// ── Wiki: get_item ────────────────────────────────────────────────────────────

#[tokio::test]
async fn get_item_by_id() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/item/id/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": 42})))
        .expect(1)
        .mount(&mock)
        .await;

    let result = server
        .get_item(Parameters(ItemIdParam { id: 42 }))
        .await
        .unwrap();
    assert!(text_of(result).contains("42"));
}

#[tokio::test]
async fn get_item_not_found_returns_invalid_params() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/item/id/999"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock)
        .await;

    let err = server
        .get_item(Parameters(ItemIdParam { id: 999 }))
        .await
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::INVALID_PARAMS);
}

#[tokio::test]
async fn get_item_server_error_returns_internal_error() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/item/id/1"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock)
        .await;

    let err = server
        .get_item(Parameters(ItemIdParam { id: 1 }))
        .await
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::INTERNAL_ERROR);
}

// ── Wiki: get_item_by_title ───────────────────────────────────────────────────

#[tokio::test]
async fn get_item_by_title_url_encodes() {
    let (server, mock) = make_server().await;
    // "A/B Test" → "A%2FB%20Test"
    Mock::given(method("GET"))
        .and(path("/item/title/A%2FB%20Test"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": 5})))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .get_item_by_title(Parameters(GetItemByTitleParams {
            title: "A/B Test".to_string(),
        }))
        .await
        .unwrap();
}

// ── Wiki: get_children / get_related_items ────────────────────────────────────

#[tokio::test]
async fn get_children_uses_id() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/item/children/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .get_children(Parameters(ItemIdParam { id: 7 }))
        .await
        .unwrap();
}

#[tokio::test]
async fn get_related_items_uses_id() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/item/related/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .get_related_items(Parameters(ItemIdParam { id: 7 }))
        .await
        .unwrap();
}

// ── Wiki: create_item ─────────────────────────────────────────────────────────

#[tokio::test]
async fn create_item_posts_body() {
    let (server, mock) = make_server().await;
    Mock::given(method("POST"))
        .and(path("/item"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": 10})))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .create_item(Parameters(CreateItemParams {
            title: "New Item".to_string(),
            content: "Some content".to_string(),
            attributes: None,
        }))
        .await
        .unwrap();
}

#[tokio::test]
async fn create_item_with_attributes() {
    let (server, mock) = make_server().await;
    Mock::given(method("POST"))
        .and(path("/item"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": 11})))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .create_item(Parameters(CreateItemParams {
            title: "Item with attrs".to_string(),
            content: "body".to_string(),
            attributes: Some(json!({"priority": "high"})),
        }))
        .await
        .unwrap();
}

// ── Wiki: update_item ─────────────────────────────────────────────────────────

#[tokio::test]
async fn update_item_patches_path() {
    let (server, mock) = make_server().await;
    Mock::given(method("PATCH"))
        .and(path("/item/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": 5})))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .update_item(Parameters(UpdateItemParams {
            id: 5,
            title: Some("Updated".to_string()),
            content: None,
        }))
        .await
        .unwrap();
}

// ── Wiki: delete_item ─────────────────────────────────────────────────────────

#[tokio::test]
async fn delete_item_correct_path() {
    let (server, mock) = make_server().await;
    Mock::given(method("DELETE"))
        .and(path("/item/3"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .delete_item(Parameters(ItemIdParam { id: 3 }))
        .await
        .unwrap();
}

// ── Wiki: set_item_attributes ─────────────────────────────────────────────────

#[tokio::test]
async fn set_item_attributes_patches_attr_path() {
    let (server, mock) = make_server().await;
    Mock::given(method("PATCH"))
        .and(path("/item/5/attr"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .set_item_attributes(Parameters(SetAttributesParams {
            id: 5,
            attributes: json!({"priority": "high"}),
        }))
        .await
        .unwrap();
}

// ── Wiki: delete_item_attribute ───────────────────────────────────────────────

#[tokio::test]
async fn delete_item_attribute_url_encodes_key() {
    let (server, mock) = make_server().await;
    Mock::given(method("DELETE"))
        .and(path("/item/5/attr/due%20date"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .delete_item_attribute(Parameters(DeleteAttributeParams {
            id: 5,
            key: "due date".to_string(),
        }))
        .await
        .unwrap();
}

// ── Wiki: assign / unassign item type ─────────────────────────────────────────

#[tokio::test]
async fn assign_item_type_url_encodes() {
    let (server, mock) = make_server().await;
    Mock::given(method("POST"))
        .and(path("/item/7/assign_type/My%20Goal"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .assign_item_type(Parameters(AssignTypeParams {
            item_id: 7,
            type_name: "My Goal".to_string(),
        }))
        .await
        .unwrap();
}

#[tokio::test]
async fn unassign_item_type_url_encodes() {
    let (server, mock) = make_server().await;
    Mock::given(method("DELETE"))
        .and(path("/item/7/assign_type/My%20Goal"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .unassign_item_type(Parameters(AssignTypeParams {
            item_id: 7,
            type_name: "My Goal".to_string(),
        }))
        .await
        .unwrap();
}

// ── Planner: get_day_plan ─────────────────────────────────────────────────────

#[tokio::test]
async fn get_day_plan_correct_path() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/planner/day/2026-05-20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .get_day_plan(Parameters(DateParam { date: "2026-05-20".to_string() }))
        .await
        .unwrap();
}

#[tokio::test]
async fn get_day_plan_not_found_returns_invalid_params() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/planner/day/2026-05-20"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock)
        .await;

    let err = server
        .get_day_plan(Parameters(DateParam { date: "2026-05-20".to_string() }))
        .await
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::INVALID_PARAMS);
}

// ── Planner: get_week_plan ────────────────────────────────────────────────────

#[tokio::test]
async fn get_week_plan_correct_path() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/planner/week/2026-W21"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .get_week_plan(Parameters(WeekParam { week: "2026-W21".to_string() }))
        .await
        .unwrap();
}

// ── Planner: get_month_plan ───────────────────────────────────────────────────

#[tokio::test]
async fn get_month_plan_correct_path() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/planner/month/5/year/2026"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .get_month_plan(Parameters(MonthYearParam { month: 5, year: 2026 }))
        .await
        .unwrap();
}

// ── Planner: get_repeat_entries ───────────────────────────────────────────────

#[tokio::test]
async fn get_repeat_entries_correct_path() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/repeat/day/2026-05-20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .get_repeat_entries(Parameters(DateParam { date: "2026-05-20".to_string() }))
        .await
        .unwrap();
}

// ── Planner: update_repeat_status ────────────────────────────────────────────

#[tokio::test]
async fn update_repeat_status_puts_correct_path() {
    let (server, mock) = make_server().await;
    Mock::given(method("PUT"))
        .and(path("/repeat/status"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .update_repeat_status(Parameters(UpdateRepeatParams {
            item_id: 42,
            date: "2026-05-20".to_string(),
            status: Some("Complete".to_string()),
            comment: None,
        }))
        .await
        .unwrap();
}

// ── Planner: comments ─────────────────────────────────────────────────────────

#[tokio::test]
async fn get_comments_for_item_correct_path() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/comment/item/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .get_comments_for_item(Parameters(PlannerItemIdParam { item_id: 42 }))
        .await
        .unwrap();
}

#[tokio::test]
async fn get_comments_for_day_correct_path() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/comment/day/2026-05-20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .get_comments_for_day(Parameters(DateParam { date: "2026-05-20".to_string() }))
        .await
        .unwrap();
}

#[tokio::test]
async fn add_comment_posts_body() {
    let (server, mock) = make_server().await;
    Mock::given(method("POST"))
        .and(path("/comment"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": 1})))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .add_comment(Parameters(AddCommentParams {
            item_id: 42,
            timestamp: "2026-05-20 10:00:00".to_string(),
            content: "A note".to_string(),
        }))
        .await
        .unwrap();
}

#[tokio::test]
async fn update_comment_patches_path() {
    let (server, mock) = make_server().await;
    Mock::given(method("PATCH"))
        .and(path("/comment/9"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": 9})))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .update_comment(Parameters(UpdateCommentParams {
            comment_id: 9,
            content: "Updated note".to_string(),
        }))
        .await
        .unwrap();
}

#[tokio::test]
async fn delete_comment_correct_path() {
    let (server, mock) = make_server().await;
    Mock::given(method("DELETE"))
        .and(path("/comment/9"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .delete_comment(Parameters(CommentIdParam { comment_id: 9 }))
        .await
        .unwrap();
}

// ── Automation: Rules ─────────────────────────────────────────────────────────

#[tokio::test]
async fn list_rules_correct_path() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/rule"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .expect(1)
        .mount(&mock)
        .await;

    server.list_rules().await.unwrap();
}

#[tokio::test]
async fn get_rule_correct_path() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/rule/3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": 3})))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .get_rule(Parameters(RuleIdParam { rule_id: 3 }))
        .await
        .unwrap();
}

#[tokio::test]
async fn create_rule_posts_body() {
    let (server, mock) = make_server().await;
    Mock::given(method("POST"))
        .and(path("/rule"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": 1})))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .create_rule(Parameters(CreateRuleParams {
            name: "My Rule".to_string(),
            description: None,
            trigger: JsonObject(json!({"kind": "manual"})),
            script: "zealot.log('hi')".to_string(),
            enabled: Some(true),
        }))
        .await
        .unwrap();
}

#[tokio::test]
async fn create_rule_defaults_enabled_to_true() {
    let (server, mock) = make_server().await;
    Mock::given(method("POST"))
        .and(path("/rule"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": 2})))
        .expect(1)
        .mount(&mock)
        .await;

    // enabled: None should default to true in the request body
    server
        .create_rule(Parameters(CreateRuleParams {
            name: "Auto Rule".to_string(),
            description: None,
            trigger: JsonObject(json!({"kind": "manual"})),
            script: "".to_string(),
            enabled: None,
        }))
        .await
        .unwrap();
}

#[tokio::test]
async fn update_rule_patches_path() {
    let (server, mock) = make_server().await;
    Mock::given(method("PATCH"))
        .and(path("/rule/3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": 3})))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .update_rule(Parameters(UpdateRuleParams {
            rule_id: 3,
            name: Some("Renamed".to_string()),
            description: None,
            trigger: None,
            script: None,
            enabled: None,
        }))
        .await
        .unwrap();
}

#[tokio::test]
async fn delete_rule_correct_path() {
    let (server, mock) = make_server().await;
    Mock::given(method("DELETE"))
        .and(path("/rule/3"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .delete_rule(Parameters(RuleIdParam { rule_id: 3 }))
        .await
        .unwrap();
}

#[tokio::test]
async fn run_rule_posts_to_run_path() {
    let (server, mock) = make_server().await;
    Mock::given(method("POST"))
        .and(path("/rule/3/run"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"output": "ok"})))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .run_rule(Parameters(RuleIdParam { rule_id: 3 }))
        .await
        .unwrap();
}

// ── Automation: Item Types ────────────────────────────────────────────────────

#[tokio::test]
async fn list_item_types_correct_path() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/item_type"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .expect(1)
        .mount(&mock)
        .await;

    server.list_item_types().await.unwrap();
}

#[tokio::test]
async fn get_item_type_by_name_url_encodes() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/item_type/name/My%20Type"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"name": "My Type"})))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .get_item_type_by_name(Parameters(ItemTypeNameParam {
            name: "My Type".to_string(),
        }))
        .await
        .unwrap();
}

#[tokio::test]
async fn create_item_type_posts_body() {
    let (server, mock) = make_server().await;
    Mock::given(method("POST"))
        .and(path("/item_type"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": 1})))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .create_item_type(Parameters(CreateItemTypeParams {
            name: "Goal".to_string(),
            description: None,
            icon: Some("star".to_string()),
            color: Some("#4A90E2".to_string()),
        }))
        .await
        .unwrap();
}

#[tokio::test]
async fn update_item_type_patches_path() {
    let (server, mock) = make_server().await;
    Mock::given(method("PATCH"))
        .and(path("/item_type/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": 5})))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .update_item_type(Parameters(UpdateItemTypeParams {
            type_id: 5,
            name: Some("Renamed".to_string()),
            description: None,
            icon: None,
            color: None,
        }))
        .await
        .unwrap();
}

#[tokio::test]
async fn delete_item_type_correct_path() {
    let (server, mock) = make_server().await;
    Mock::given(method("DELETE"))
        .and(path("/item_type/5"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .delete_item_type(Parameters(ItemTypeIdParam { type_id: 5 }))
        .await
        .unwrap();
}

// ── Automation: Attribute Kinds ───────────────────────────────────────────────

#[tokio::test]
async fn list_attribute_kinds_correct_path() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/attribute"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .expect(1)
        .mount(&mock)
        .await;

    server.list_attribute_kinds().await.unwrap();
}

#[tokio::test]
async fn get_attribute_by_key_url_encodes() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/attribute/key/due%20date"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"key": "due date"})))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .get_attribute_by_key(Parameters(AttributeKeyParam {
            key: "due date".to_string(),
        }))
        .await
        .unwrap();
}

#[tokio::test]
async fn create_attribute_kind_default_config() {
    let (server, mock) = make_server().await;
    Mock::given(method("POST"))
        .and(path("/attribute"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": 1})))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .create_attribute_kind(Parameters(CreateAttributeKindParams {
            key: "priority".to_string(),
            description: None,
            base_type: "text".to_string(),
            config: None,
        }))
        .await
        .unwrap();
}

#[tokio::test]
async fn update_attribute_kind_patches_path() {
    let (server, mock) = make_server().await;
    Mock::given(method("PATCH"))
        .and(path("/attribute/id/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": 7})))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .update_attribute_kind(Parameters(UpdateAttributeKindParams {
            kind_id: 7,
            description: Some("Updated desc".to_string()),
            config: None,
        }))
        .await
        .unwrap();
}

// ── Media ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn list_media_root() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/media/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .list_media(Parameters(MediaPathParam { path: None }))
        .await
        .unwrap();
}

#[tokio::test]
async fn list_media_subdir() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/media/images/2025"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .list_media(Parameters(MediaPathParam {
            path: Some("images/2025".to_string()),
        }))
        .await
        .unwrap();
}

#[tokio::test]
async fn list_media_strips_leading_slash() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/media/images"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .list_media(Parameters(MediaPathParam {
            path: Some("/images".to_string()),
        }))
        .await
        .unwrap();
}

#[tokio::test]
async fn create_media_folder_posts_body() {
    let (server, mock) = make_server().await;
    Mock::given(method("POST"))
        .and(path("/media/mkdir"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .create_media_folder(Parameters(CreateFolderParams {
            folder: "images/2025".to_string(),
        }))
        .await
        .unwrap();
}

#[tokio::test]
async fn delete_media_correct_path() {
    let (server, mock) = make_server().await;
    Mock::given(method("DELETE"))
        .and(path("/media/images/photo.jpg"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .delete_media(Parameters(DeleteMediaParams {
            path: "images/photo.jpg".to_string(),
        }))
        .await
        .unwrap();
}

#[tokio::test]
async fn delete_media_strips_leading_slash() {
    let (server, mock) = make_server().await;
    Mock::given(method("DELETE"))
        .and(path("/media/images/photo.jpg"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .delete_media(Parameters(DeleteMediaParams {
            path: "/images/photo.jpg".to_string(),
        }))
        .await
        .unwrap();
}

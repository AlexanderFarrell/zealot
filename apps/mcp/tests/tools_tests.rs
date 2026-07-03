use std::collections::HashMap;

use reqwest::StatusCode;
use rmcp::{handler::server::wrapper::Parameters, model::ErrorCode};
use serde_json::{Value, json};
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use zealot_mcp::{
    client::{ApiError, ZealotClient},
    output::Detail,
    tools::{
        ZealotServer,
        analysis::{HabitStatsParams, LimitParams},
        err_ctx,
        media::GetMediaParams,
        planner::{AddJournalEntryParams, DayDashboardParams, GetPlanParams},
        time_block::CreateTimeBlockParams,
        wiki::{
            AppendToItemParams, BrowseItemsParams, BrowseMode, CreateItemParams, ItemRefParam,
            LinkDirection, LinkParam, LinkedItemsParams, SearchItemsParams,
        },
    },
};

async fn make_server() -> (ZealotServer, MockServer) {
    let mock = MockServer::start().await;
    let server = ZealotServer {
        client: ZealotClient::new(mock.uri(), "key"),
        journal_item: None,
    };
    (server, mock)
}

async fn make_server_with_journal() -> (ZealotServer, MockServer) {
    let mock = MockServer::start().await;
    let server = ZealotServer {
        client: ZealotClient::new(mock.uri(), "key"),
        journal_item: Some("Journal".to_string()),
    };
    (server, mock)
}

fn text_of(result: rmcp::model::CallToolResult) -> String {
    result.content[0].as_text().unwrap().text.clone()
}

fn item(id: i64, title: &str, content: &str) -> Value {
    json!({
        "item_id": id,
        "title": title,
        "content": content,
        "attributes": {},
        "types": [],
        "links": []
    })
}

fn item_with_links(id: i64, title: &str, links: Value) -> Value {
    json!({
        "item_id": id,
        "title": title,
        "content": "",
        "attributes": {},
        "types": [],
        "links": links
    })
}

fn repeat_entry(id: i64, title: &str, date: &str, status: &str) -> Value {
    json!({
        "item": item(id, title, ""),
        "date": date,
        "status": status,
        "comment": ""
    })
}

fn time_block(id: i64, item_id: i64, title: &str) -> Value {
    json!({
        "block_id": id,
        "item": item(item_id, title, ""),
        "date": "2026-07-03",
        "start_min": 540,
        "end_min": 600,
        "note": "Focus"
    })
}

fn comment(id: i64, item_id: i64, title: &str) -> Value {
    json!({
        "comment_id": id,
        "item": item(item_id, title, ""),
        "timestamp": "2026-07-03 09:00:00",
        "content": "Logged"
    })
}

#[test]
fn err_ctx_maps_client_and_server_errors() {
    let not_found = err_ctx("item #1", ApiError::NotFound);
    assert_eq!(not_found.code, ErrorCode::INVALID_PARAMS);
    assert!(not_found.message.contains("not found"));

    let client = err_ctx(
        "item",
        ApiError::Http {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            message: "bad attrs".to_string(),
        },
    );
    assert_eq!(client.code, ErrorCode::INVALID_PARAMS);
    assert!(client.message.contains("bad attrs"));

    let server = err_ctx(
        "item",
        ApiError::Http {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "boom".to_string(),
        },
    );
    assert_eq!(server.code, ErrorCode::INTERNAL_ERROR);
    assert!(server.message.contains("boom"));
}

#[tokio::test]
async fn get_item_accepts_hash_id_and_projects_summary() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/item/id/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(item(42, "Alpha", "hello world")))
        .expect(1)
        .mount(&mock)
        .await;

    let result = server
        .get_item(Parameters(ItemRefParam {
            item: "#42".to_string(),
            detail: Detail::Summary,
        }))
        .await
        .unwrap();
    let value: Value = serde_json::from_str(&text_of(result)).unwrap();
    assert_eq!(value["item_id"], 42);
    assert_eq!(value["preview"], "hello world");
    assert!(value.get("content").is_none());
}

#[tokio::test]
async fn get_item_accepts_exact_title_and_url_encodes_path_segment() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/item/title/A%2FB%20Test"))
        .respond_with(ResponseTemplate::new(200).set_body_json(item(5, "A/B Test", "")))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .get_item(Parameters(ItemRefParam {
            item: "A/B Test".to_string(),
            detail: Detail::Full,
        }))
        .await
        .unwrap();
}

#[tokio::test]
async fn browse_items_recent_returns_summary_page() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/item/recent"))
        .and(query_param("limit", "2"))
        .and(query_param("offset", "0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            item(1, "One", "body one"),
            item(2, "Two", "body two")
        ])))
        .expect(1)
        .mount(&mock)
        .await;

    let result = server
        .browse_items(Parameters(BrowseItemsParams {
            mode: BrowseMode::Recent,
            type_filter: None,
            limit: Some(2),
            offset: None,
            detail: Detail::Summary,
        }))
        .await
        .unwrap();
    let value: Value = serde_json::from_str(&text_of(result)).unwrap();
    assert_eq!(value["count"], 2);
    assert_eq!(value["next_offset"], 2);
    assert_eq!(value["items"][0]["preview"], "body one");
    assert!(value["items"][0].get("content").is_none());
}

#[tokio::test]
async fn search_items_passes_scope_regex_and_pagination() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/item/search"))
        .and(query_param("term", "hello world"))
        .and(query_param("scope", "content"))
        .and(query_param("regex", "true"))
        .and(query_param("limit", "5"))
        .and(query_param("offset", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([{
            "item_id": 1,
            "title": "Hit",
            "content": "matching content",
            "attributes": {},
            "types": [],
            "links": [],
            "match_scope": "content",
            "snippet": "matching"
        }])))
        .expect(1)
        .mount(&mock)
        .await;

    let result = server
        .search_items(Parameters(SearchItemsParams {
            term: "hello world".to_string(),
            limit: Some(5),
            offset: Some(10),
            scope: Some("content".to_string()),
            regex: Some(true),
            detail: Detail::Summary,
        }))
        .await
        .unwrap();
    let value: Value = serde_json::from_str(&text_of(result)).unwrap();
    assert_eq!(value["items"][0]["match_scope"], "content");
    assert_eq!(value["items"][0]["snippet"], "matching");
}

#[tokio::test]
async fn get_linked_items_supports_backlinks_after_title_resolution() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/item/title/Target"))
        .respond_with(ResponseTemplate::new(200).set_body_json(item(7, "Target", "")))
        .expect(1)
        .mount(&mock)
        .await;
    Mock::given(method("GET"))
        .and(path("/item/backlinks/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([item(8, "Source", "")])))
        .expect(1)
        .mount(&mock)
        .await;

    let result = server
        .get_linked_items(Parameters(LinkedItemsParams {
            item: "Target".to_string(),
            direction: LinkDirection::Backlinks,
            detail: Detail::Meta,
            limit: None,
            offset: None,
        }))
        .await
        .unwrap();
    let value: Value = serde_json::from_str(&text_of(result)).unwrap();
    assert_eq!(value["items"][0]["title"], "Source");
}

#[tokio::test]
async fn create_item_resolves_parent_and_posts_full_create_body() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/item/title/Parent%20Title"))
        .respond_with(ResponseTemplate::new(200).set_body_json(item(9, "Parent Title", "")))
        .expect(1)
        .mount(&mock)
        .await;
    Mock::given(method("POST"))
        .and(path("/item"))
        .and(body_json(json!({
            "title": "Child",
            "content": "Body",
            "attributes": {"Parent": 9, "priority": "high"},
            "types": ["Task"],
            "links": [{"other_item_id": 1, "relationship": "blocks"}]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(item(10, "Child", "Body")))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .create_item(Parameters(CreateItemParams {
            title: "Child".to_string(),
            content: Some("Body".to_string()),
            attributes: Some(HashMap::from([("priority".to_string(), json!("high"))])),
            types: Some(vec!["Task".to_string()]),
            links: Some(vec![LinkParam {
                other_item_id: 1,
                relationship: "blocks".to_string(),
            }]),
            parent: Some("Parent Title".to_string()),
        }))
        .await
        .unwrap();
}

#[tokio::test]
async fn append_to_item_fetches_then_patches_newline_joined_content() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/item/id/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(item(5, "Note", "one")))
        .expect(1)
        .mount(&mock)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/item/5"))
        .and(body_json(json!({
            "item_id": 5,
            "content": "one\ntwo\n"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(item(5, "Note", "one\ntwo\n")))
        .expect(1)
        .mount(&mock)
        .await;

    let result = server
        .append_to_item(Parameters(AppendToItemParams {
            item: "5".to_string(),
            text: "two".to_string(),
        }))
        .await
        .unwrap();
    let value: Value = serde_json::from_str(&text_of(result)).unwrap();
    assert_eq!(value["content_chars"], 8);
}

#[tokio::test]
async fn day_dashboard_hits_plan_habits_blocks_and_comments_once() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/planner/day/2026-07-03"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([item(1, "Plan", "Do it")])))
        .expect(1)
        .mount(&mock)
        .await;
    Mock::given(method("GET"))
        .and(path("/repeat/day/2026-07-03"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!([repeat_entry(
                2,
                "Habit",
                "2026-07-03",
                "Complete"
            )])),
        )
        .expect(1)
        .mount(&mock)
        .await;
    Mock::given(method("GET"))
        .and(path("/time_block/day/2026-07-03"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([time_block(3, 1, "Plan")])))
        .expect(1)
        .mount(&mock)
        .await;
    Mock::given(method("GET"))
        .and(path("/comment/day/2026-07-03"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([comment(4, 1, "Plan")])))
        .expect(1)
        .mount(&mock)
        .await;

    let result = server
        .day_dashboard(Parameters(DayDashboardParams {
            date: Some("2026-07-03".to_string()),
        }))
        .await
        .unwrap();
    let value: Value = serde_json::from_str(&text_of(result)).unwrap();
    assert_eq!(value["date"], "2026-07-03");
    assert_eq!(value["habits"][0]["status"], "Complete");
    assert_eq!(value["time_blocks"][0]["start"], "9:00");
    assert_eq!(value["journal"][0]["content"], "Logged");
}

#[tokio::test]
async fn get_plan_supports_year_period() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/planner/year/2026"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([item(1, "Yearly", "")])))
        .expect(1)
        .mount(&mock)
        .await;

    let result = server
        .get_plan(Parameters(GetPlanParams {
            period: "2026".to_string(),
            detail: Detail::Meta,
            limit: None,
            offset: None,
        }))
        .await
        .unwrap();
    let value: Value = serde_json::from_str(&text_of(result)).unwrap();
    assert_eq!(value["items"][0]["title"], "Yearly");
}

#[tokio::test]
async fn create_time_block_accepts_clock_strings() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/item/id/7"))
        .respond_with(ResponseTemplate::new(200).set_body_json(item(7, "Focus", "")))
        .expect(1)
        .mount(&mock)
        .await;
    Mock::given(method("POST"))
        .and(path("/time_block"))
        .and(body_json(json!({
            "item_id": 7,
            "date": "2026-07-03",
            "start_min": 570,
            "end_min": 645,
            "note": "Deep work"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "block_id": 99,
            "item": item(7, "Focus", ""),
            "date": "2026-07-03",
            "start_min": 570,
            "end_min": 645,
            "note": "Deep work"
        })))
        .expect(1)
        .mount(&mock)
        .await;

    let result = server
        .create_time_block(Parameters(CreateTimeBlockParams {
            item: "#7".to_string(),
            date: "2026-07-03".to_string(),
            start: "9:30".to_string(),
            end: "10:45".to_string(),
            note: Some("Deep work".to_string()),
        }))
        .await
        .unwrap();
    let value: Value = serde_json::from_str(&text_of(result)).unwrap();
    assert_eq!(value["start"], "9:30");
    assert_eq!(value["end"], "10:45");
}

#[tokio::test]
async fn add_journal_entry_uses_configured_item_ref() {
    let (server, mock) = make_server_with_journal().await;
    Mock::given(method("GET"))
        .and(path("/item/title/Journal"))
        .respond_with(ResponseTemplate::new(200).set_body_json(item(12, "Journal", "")))
        .expect(1)
        .mount(&mock)
        .await;
    Mock::given(method("POST"))
        .and(path("/comment"))
        .respond_with(ResponseTemplate::new(200).set_body_json(comment(13, 12, "Journal")))
        .expect(1)
        .mount(&mock)
        .await;

    let result = server
        .add_journal_entry(Parameters(AddJournalEntryParams {
            content: "Logged".to_string(),
        }))
        .await
        .unwrap();
    let value: Value = serde_json::from_str(&text_of(result)).unwrap();
    assert_eq!(value["item"]["item_id"], 12);
}

#[tokio::test]
async fn get_media_returns_text_content_inline() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/media/notes/today.txt"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string("hello")
                .insert_header("content-type", "text/plain"),
        )
        .expect(1)
        .mount(&mock)
        .await;

    let result = server
        .get_media(Parameters(GetMediaParams {
            path: "notes/today.txt".to_string(),
        }))
        .await
        .unwrap();
    assert_eq!(text_of(result), "hello");
}

#[tokio::test]
async fn orphaned_items_pages_recent_items_and_uses_link_graph() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/item/recent"))
        .and(query_param("limit", "100"))
        .and(query_param("offset", "0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            item_with_links(
                1,
                "Has outgoing",
                json!([{"other_item_id": 2, "relationship": "topic"}])
            ),
            item(2, "Has incoming", ""),
            item(3, "Orphan", "")
        ])))
        .expect(1)
        .mount(&mock)
        .await;

    let result = server
        .orphaned_items(Parameters(LimitParams { limit: Some(10) }))
        .await
        .unwrap();
    let value: Value = serde_json::from_str(&text_of(result)).unwrap();
    assert_eq!(value["matching_count"], 1);
    assert_eq!(value["items"][0]["title"], "Orphan");
}

#[tokio::test]
async fn habit_stats_computes_neutral_status_streaks() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/repeat/range"))
        .and(query_param("start", "2026-07-01"))
        .and(query_param("end", "2026-07-05"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            repeat_entry(1, "Meditate", "2026-07-01", "Complete"),
            repeat_entry(1, "Meditate", "2026-07-02", "Skip"),
            repeat_entry(1, "Meditate", "2026-07-03", "Complete"),
            repeat_entry(1, "Meditate", "2026-07-04", "Not Complete"),
            repeat_entry(1, "Meditate", "2026-07-05", "Complete")
        ])))
        .expect(1)
        .mount(&mock)
        .await;

    let result = server
        .habit_stats(Parameters(HabitStatsParams {
            start_date: "2026-07-01".to_string(),
            end_date: "2026-07-05".to_string(),
        }))
        .await
        .unwrap();
    let value: Value = serde_json::from_str(&text_of(result)).unwrap();
    let habit = &value["habits"][0];
    assert_eq!(habit["counts"]["complete"], 3);
    assert_eq!(habit["counts"]["skip"], 1);
    assert_eq!(habit["counts"]["not_complete"], 1);
    assert_eq!(habit["longest_streak"], 2);
    assert_eq!(habit["current_streak"], 1);
    assert_eq!(habit["completion_rate"], 0.75);
}

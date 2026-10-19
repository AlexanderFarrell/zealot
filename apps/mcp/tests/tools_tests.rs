use std::collections::HashMap;

use reqwest::StatusCode;
use rmcp::{handler::server::wrapper::Parameters, model::ErrorCode};
use serde_json::{Value, json};
use wiremock::matchers::{body_json, method, path, path_regex, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

use zealot_mcp::{
    client::{ApiError, ZealotClient},
    output::Detail,
    tools::{
        ZealotServer,
        analysis::{HabitStatsParams, LimitParams, WikiStatsParams},
        automation::{DeleteAttributeKindParams, ItemTypeNameParam, ListAttributeKindsParams},
        err_ctx,
        media::GetMediaParams,
        planner::{AddCommentParams, AddJournalEntryParams, DayDashboardParams, GetPlanParams},
        statistic::{RecordStatisticEntryParams, StatisticQueryParams},
        time_block::CreateTimeBlockParams,
        view::{RunViewParams, ValidateViewParams},
        wiki::{
            AppendToItemParams, AttributeFilterParam, BrowseItemsParams, BrowseMode,
            CreateItemParams, FilterItemsParams, ItemRefParam, LinkDirection, LinkParam,
            LinkedItemsParams, SearchItemsParams, SetStatusParams, SortParam, UpdateItemParams,
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
            include_archived: false,
            mode: BrowseMode::Recent,
            type_filter: None,
            sort: None,
            limit: Some(2),
            offset: None,
            detail: Detail::Summary,
            fields: None,
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
            include_archived: false,
            term: "hello world".to_string(),
            limit: Some(5),
            offset: Some(10),
            scope: Some("content".to_string()),
            regex: Some(true),
            detail: Detail::Summary,
            fields: None,
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
            include_archived: false,
            item: "Target".to_string(),
            direction: LinkDirection::Backlinks,
            detail: Detail::Meta,
            fields: None,
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
            "attributes": {"Parent": [9], "Priority": "high"},
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
            attributes: Some(HashMap::from([("Priority".to_string(), json!("high"))])),
            types: Some(vec!["Task".to_string()]),
            links: Some(vec![LinkParam {
                other_item_id: 1,
                relationship: "blocks".to_string(),
            }]),
            parent: Some("Parent Title".to_string()),
            full: None,
        }))
        .await
        .unwrap();
}

#[tokio::test]
async fn create_item_merges_parent_with_parents_in_attributes() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/item/id/11"))
        .respond_with(ResponseTemplate::new(200).set_body_json(item(11, "Zealot", "")))
        .expect(1)
        .mount(&mock)
        .await;
    Mock::given(method("POST"))
        .and(path("/item"))
        .and(body_json(json!({
            "title": "Child",
            "content": "",
            "attributes": {"Parent": [4, 11]}
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(item(12, "Child", "")))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .create_item(Parameters(CreateItemParams {
            title: "Child".to_string(),
            content: None,
            attributes: Some(HashMap::from([("Parent".to_string(), json!([4, 11]))])),
            types: None,
            links: None,
            parent: Some("#11".to_string()),
            full: None,
        }))
        .await
        .unwrap();
}

#[tokio::test]
async fn create_item_wraps_scalar_parent_attribute_before_adding_parent() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/item/id/11"))
        .respond_with(ResponseTemplate::new(200).set_body_json(item(11, "Zealot", "")))
        .expect(1)
        .mount(&mock)
        .await;
    Mock::given(method("POST"))
        .and(path("/item"))
        .and(body_json(json!({
            "title": "Child",
            "content": "",
            "attributes": {"Parent": [4, 11]}
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(item(12, "Child", "")))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .create_item(Parameters(CreateItemParams {
            title: "Child".to_string(),
            content: None,
            attributes: Some(HashMap::from([("Parent".to_string(), json!(4))])),
            types: None,
            links: None,
            parent: Some("11".to_string()),
            full: None,
        }))
        .await
        .unwrap();
}

#[tokio::test]
async fn append_to_item_posts_a_server_side_append() {
    let (server, mock) = make_server().await;
    Mock::given(method("POST"))
        .and(path("/item/5/content"))
        .and(body_json(json!({"op": "append", "text": "two"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "item_id": 5,
            "title": "Note",
            "version": "abc",
            "content_chars": 8,
            "changed_lines": {"start": 2, "removed": 0, "added": 1}
        })))
        .expect(1)
        .mount(&mock)
        .await;

    let result = server
        .append_to_item(Parameters(AppendToItemParams {
            item: "5".to_string(),
            text: "two".to_string(),
            expected_version: None,
        }))
        .await
        .unwrap();
    let value: Value = serde_json::from_str(&text_of(result)).unwrap();
    assert_eq!(value["content_chars"], 8);
    assert_eq!(value["version"], "abc");
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
            include_archived: false,
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
    Mock::given(method("POST"))
        .and(path("/item/query"))
        .and(body_json(json!({
            "include_archived": false, "types": [], "type_mode": "", "under": null,
            "filters": [{"key": "Year", "op": "eq", "value": 2026, "list_mode": "any"}],
            "sort": null, "limit": 50, "offset": 0
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "items": [item(1, "Yearly", "")], "total": 1
        })))
        .expect(1)
        .mount(&mock)
        .await;

    let result = server
        .get_plan(Parameters(GetPlanParams {
            include_archived: false,
            period: "2026".to_string(),
            detail: Detail::Meta,
            fields: None,
            limit: None,
            offset: None,
        }))
        .await
        .unwrap();
    let value: Value = serde_json::from_str(&text_of(result)).unwrap();
    assert_eq!(value["items"][0]["title"], "Yearly");
    assert_eq!(value["total"], 1);
    assert!(value.get("next_offset").is_none());
}

#[tokio::test]
async fn get_plan_month_pages_server_side() {
    let (server, mock) = make_server().await;
    Mock::given(method("POST"))
        .and(path("/item/query"))
        .and(body_json(json!({
            "include_archived": false, "types": [], "type_mode": "", "under": null,
            "filters": [
                {"key": "Month", "op": "eq", "value": 10, "list_mode": "any"},
                {"key": "Year", "op": "eq", "value": 2026, "list_mode": "any"}
            ],
            "sort": null, "limit": 1, "offset": 1
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "items": [item(2, "Second", "")], "total": 5
        })))
        .expect(1)
        .mount(&mock)
        .await;

    let result = server
        .get_plan(Parameters(GetPlanParams {
            include_archived: false,
            period: "2026-10".to_string(),
            detail: Detail::Meta,
            fields: None,
            limit: Some(1),
            offset: Some(1),
        }))
        .await
        .unwrap();
    let value: Value = serde_json::from_str(&text_of(result)).unwrap();
    assert_eq!(value["total"], 5);
    assert_eq!(value["next_offset"], 2);
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
    assert_eq!(value["item_id"], 12);
    assert!(value.get("item").is_none());
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

#[tokio::test]
async fn habit_stats_counts_only_through_today() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/repeat/range"))
        .and(query_param("start", "2026-07-01"))
        .and(query_param("end", "2026-07-04"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            repeat_entry(1, "Meditate", "2026-07-01", "Complete"),
            repeat_entry(1, "Meditate", "2026-07-02", "Not Complete"),
            repeat_entry(1, "Meditate", "2026-07-03", "Complete"),
            repeat_entry(1, "Meditate", "2026-07-04", "Not Complete"),
            repeat_entry(2, "Stretch", "2026-07-03", "Complete"),
            repeat_entry(2, "Stretch", "2026-07-04", "Complete")
        ])))
        .expect(1)
        .mount(&mock)
        .await;

    let today = chrono::NaiveDate::from_ymd_opt(2026, 7, 4).unwrap();
    let result = server
        .habit_stats_as_of(
            HabitStatsParams {
                start_date: "2026-07-01".to_string(),
                end_date: "2026-07-06".to_string(),
            },
            today,
        )
        .await
        .unwrap();
    let value: Value = serde_json::from_str(&text_of(result)).unwrap();
    assert_eq!(value["end_date"], "2026-07-06");
    assert_eq!(value["counted_through"], "2026-07-04");

    // Unlogged past day counts and breaks the streak; unlogged today is pending.
    let meditate = &value["habits"][0];
    assert_eq!(meditate["entries"], 3);
    assert_eq!(meditate["counts"]["complete"], 2);
    assert_eq!(meditate["counts"]["not_complete"], 1);
    assert_eq!(meditate["completion_rate"], 0.6667);
    assert_eq!(meditate["current_streak"], 1);
    assert_eq!(meditate["longest_streak"], 1);

    // A completed today counts toward the rate and the streak.
    let stretch = &value["habits"][1];
    assert_eq!(stretch["counts"]["complete"], 2);
    assert_eq!(stretch["completion_rate"], 1.0);
    assert_eq!(stretch["current_streak"], 2);
}

#[tokio::test]
async fn habit_stats_counts_unlogged_day_once_it_has_passed() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/repeat/range"))
        .and(query_param("start", "2026-07-03"))
        .and(query_param("end", "2026-07-04"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            repeat_entry(1, "Meditate", "2026-07-03", "Complete"),
            repeat_entry(1, "Meditate", "2026-07-04", "Not Complete")
        ])))
        .expect(1)
        .mount(&mock)
        .await;

    let tomorrow = chrono::NaiveDate::from_ymd_opt(2026, 7, 5).unwrap();
    let result = server
        .habit_stats_as_of(
            HabitStatsParams {
                start_date: "2026-07-03".to_string(),
                end_date: "2026-07-04".to_string(),
            },
            tomorrow,
        )
        .await
        .unwrap();
    let value: Value = serde_json::from_str(&text_of(result)).unwrap();
    assert_eq!(value["counted_through"], "2026-07-04");
    let habit = &value["habits"][0];
    assert_eq!(habit["counts"]["not_complete"], 1);
    assert_eq!(habit["completion_rate"], 0.5);
    assert_eq!(habit["current_streak"], 0);
}

#[tokio::test]
async fn habit_stats_future_range_counts_nothing() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/repeat/range"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .expect(0)
        .mount(&mock)
        .await;

    let today = chrono::NaiveDate::from_ymd_opt(2026, 7, 4).unwrap();
    let result = server
        .habit_stats_as_of(
            HabitStatsParams {
                start_date: "2026-07-05".to_string(),
                end_date: "2026-07-10".to_string(),
            },
            today,
        )
        .await
        .unwrap();
    let value: Value = serde_json::from_str(&text_of(result)).unwrap();
    assert_eq!(value["counted_through"], Value::Null);
    assert_eq!(value["habit_count"], 0);
}

#[tokio::test]
async fn habit_stats_tool_stops_at_local_today() {
    let (server, mock) = make_server().await;
    let today = zealot_mcp::tools::local_today();
    let start = today - chrono::Duration::days(1);
    let end = today + chrono::Duration::days(3);
    Mock::given(method("GET"))
        .and(path("/repeat/range"))
        .and(query_param("start", start.to_string()))
        .and(query_param("end", today.to_string()))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            repeat_entry(1, "Meditate", &start.to_string(), "Complete"),
            repeat_entry(1, "Meditate", &today.to_string(), "Not Complete")
        ])))
        .expect(1)
        .mount(&mock)
        .await;

    let result = server
        .habit_stats(Parameters(HabitStatsParams {
            start_date: start.to_string(),
            end_date: end.to_string(),
        }))
        .await
        .unwrap();
    let value: Value = serde_json::from_str(&text_of(result)).unwrap();
    assert_eq!(value["counted_through"], today.to_string());
    assert_eq!(value["habits"][0]["current_streak"], 1);
    assert_eq!(value["habits"][0]["completion_rate"], 1.0);
}

#[tokio::test]
async fn record_statistic_entry_resolves_item_and_posts_typed_body() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/item/id/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(item(42, "Body mass", "")))
        .expect(1)
        .mount(&mock)
        .await;
    Mock::given(method("POST"))
        .and(path("/statistic/42/entries"))
        .and(body_json(json!({
            "value": 82.5,
            "occurred_at": "2026-08-13T08:30:00Z",
            "comment": "Morning"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "statistic_entry_id": 7,
            "item_id": 42,
            "value": 82.5,
            "occurred_at": "2026-08-13T08:30:00+00:00",
            "related_item_id": null,
            "comment": "Morning",
            "created_at": "2026-08-13T08:31:00+00:00",
            "updated_at": "2026-08-13T08:31:00+00:00"
        })))
        .expect(1)
        .mount(&mock)
        .await;

    let result = server
        .record_statistic_entry(Parameters(RecordStatisticEntryParams {
            statistic: "#42".to_string(),
            value: 82.5,
            occurred_at: Some("2026-08-13T08:30:00Z".to_string()),
            related_item: None,
            comment: Some("Morning".to_string()),
        }))
        .await
        .unwrap();
    let value: Value = serde_json::from_str(&text_of(result)).unwrap();
    assert_eq!(value["statistic_entry_id"], 7);
    assert_eq!(value["value"], 82.5);
}

#[tokio::test]
async fn get_statistic_summary_returns_period_statistics() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/item/id/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(item(42, "Body mass", "")))
        .expect(1)
        .mount(&mock)
        .await;
    Mock::given(method("GET"))
        .and(path("/statistic/42/summary"))
        .and(query_param("start", "2026-08-01T00:00:00Z"))
        .and(query_param("end", "2026-09-01T00:00:00Z"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "count": 2,
            "first": {"value": 82.5, "occurred_at": "2026-08-13T08:30:00+00:00"},
            "latest": {"value": 81.5, "occurred_at": "2026-08-14T08:30:00+00:00"},
            "minimum": 81.5,
            "maximum": 82.5,
            "average": 82.0,
            "sum": 164.0,
            "delta": -1.0
        })))
        .expect(1)
        .mount(&mock)
        .await;

    let result = server
        .get_statistic_summary(Parameters(StatisticQueryParams {
            statistic: "#42".to_string(),
            start: Some("2026-08-01T00:00:00Z".to_string()),
            end: Some("2026-09-01T00:00:00Z".to_string()),
            limit: None,
            offset: None,
        }))
        .await
        .unwrap();
    let value: Value = serde_json::from_str(&text_of(result)).unwrap();
    assert_eq!(value["count"], 2);
    assert_eq!(value["delta"], -1.0);
}

fn typed_item(id: i64, title: &str, content: &str) -> Value {
    json!({
        "item_id": id,
        "title": title,
        "content": content,
        "attributes": {"Status": "To Do"},
        "types": [{"type_id": 9, "is_system": false, "name": "Ticket"}],
        "links": []
    })
}

fn create_params(full: Option<bool>) -> CreateItemParams {
    CreateItemParams {
        title: "T".to_string(),
        content: Some("x".repeat(4000)),
        attributes: None,
        types: None,
        links: None,
        parent: None,
        full,
    }
}

#[tokio::test]
async fn create_item_returns_compact_result_by_default() {
    let (server, mock) = make_server().await;
    Mock::given(method("POST"))
        .and(path("/item"))
        .respond_with(ResponseTemplate::new(200).set_body_json(typed_item(
            20,
            "T",
            &"x".repeat(4000),
        )))
        .mount(&mock)
        .await;

    let text = text_of(
        server
            .create_item(Parameters(create_params(None)))
            .await
            .unwrap(),
    );
    assert!(text.len() < 500, "compact result was {} bytes", text.len());
    let value: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(value["item_id"], 20);
    assert_eq!(value["types"], json!(["Ticket"]));
    assert_eq!(value["attributes"]["Status"], "To Do");
    assert_eq!(value["content_chars"], 4000);
    assert!(value.get("content").is_none());
}

#[tokio::test]
async fn create_item_full_returns_complete_item() {
    let (server, mock) = make_server().await;
    Mock::given(method("POST"))
        .and(path("/item"))
        .respond_with(ResponseTemplate::new(200).set_body_json(typed_item(20, "T", "body")))
        .mount(&mock)
        .await;

    let result = server
        .create_item(Parameters(create_params(Some(true))))
        .await
        .unwrap();
    let value: Value = serde_json::from_str(&text_of(result)).unwrap();
    assert_eq!(value["content"], "body");
}

async fn mount_update(mock: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/item/id/20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(typed_item(20, "T", "old")))
        .mount(mock)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/item/20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(typed_item(20, "T2", "new body")))
        .mount(mock)
        .await;
}

#[tokio::test]
async fn update_item_compact_by_default_and_full_on_request() {
    let (server, mock) = make_server().await;
    mount_update(&mock).await;
    let params = |full| UpdateItemParams {
        item: "#20".to_string(),
        title: Some("T2".to_string()),
        content: Some("new body".to_string()),
        expected_version: None,
        full,
    };

    let value: Value = serde_json::from_str(&text_of(
        server.update_item(Parameters(params(None))).await.unwrap(),
    ))
    .unwrap();
    assert_eq!(value["title"], "T2");
    assert_eq!(value["content_chars"], 8);
    assert!(value.get("content").is_none());
    assert_eq!(
        value["version"],
        zealot_domain::item_content::content_version("new body")
    );
    assert!(value.get("changed_lines").is_some());

    let value: Value = serde_json::from_str(&text_of(
        server
            .update_item(Parameters(params(Some(true))))
            .await
            .unwrap(),
    ))
    .unwrap();
    assert_eq!(value["content"], "new body");
}

#[tokio::test]
async fn add_comment_compact_by_default_and_full_on_request() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/item/id/12"))
        .respond_with(ResponseTemplate::new(200).set_body_json(item(12, "Parent", "")))
        .mount(&mock)
        .await;
    Mock::given(method("POST"))
        .and(path("/comment"))
        .respond_with(ResponseTemplate::new(200).set_body_json(comment(13, 12, "Parent")))
        .mount(&mock)
        .await;
    let params = |full| AddCommentParams {
        item: "#12".to_string(),
        content: "hi".to_string(),
        timestamp: None,
        full,
    };

    let value: Value = serde_json::from_str(&text_of(
        server.add_comment(Parameters(params(None))).await.unwrap(),
    ))
    .unwrap();
    assert_eq!(value["comment_id"], 13);
    assert_eq!(value["item_id"], 12);
    assert!(value.get("item").is_none());
    assert!(value.get("content").is_none());

    let value: Value = serde_json::from_str(&text_of(
        server
            .add_comment(Parameters(params(Some(true))))
            .await
            .unwrap(),
    ))
    .unwrap();
    assert_eq!(value["item"]["item_id"], 12);
}

#[tokio::test]
async fn filter_items_sends_types_under_and_sort_and_reports_total() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/item/title/Zealot"))
        .respond_with(ResponseTemplate::new(200).set_body_json(item(11, "Zealot", "")))
        .expect(1)
        .mount(&mock)
        .await;
    Mock::given(method("POST"))
        .and(path("/item/query"))
        .and(body_json(json!({
            "include_archived": false, "types": ["Ticket"], "type_mode": "all", "under": 11,
            "filters": [{"key": "Status", "op": "eq", "value": "Working", "list_mode": "any"}],
            "sort": {"key": "TicketID", "direction": "desc"}, "limit": 2, "offset": 0
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "items": [item(3, "Z002", ""), item(4, "Z001", "")], "total": 7
        })))
        .expect(1)
        .mount(&mock)
        .await;

    let result = server
        .filter_items(Parameters(FilterItemsParams {
            include_archived: false,
            any_of: vec![],
            linked_from: None,
            filters: vec![AttributeFilterParam {
                key: "Status".to_string(),
                op: "eq".to_string(),
                value: json!("Working"),
                list_mode: None,
            }],
            types: Some(vec!["Ticket".to_string()]),
            type_mode: Some("all".to_string()),
            under: Some("Zealot".to_string()),
            sort: Some(SortParam {
                key: "TicketID".to_string(),
                direction: Some("desc".to_string()),
            }),
            limit: Some(2),
            offset: None,
            detail: Detail::Meta,
            fields: None,
        }))
        .await
        .unwrap();
    let value: Value = serde_json::from_str(&text_of(result)).unwrap();
    assert_eq!(value["total"], 7);
    assert_eq!(value["count"], 2);
    assert_eq!(value["next_offset"], 2);
}

#[tokio::test]
async fn filter_items_last_page_has_no_next_offset() {
    let (server, mock) = make_server().await;
    Mock::given(method("POST"))
        .and(path("/item/query"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "items": [item(3, "Z002", "")], "total": 3
        })))
        .mount(&mock)
        .await;

    let result = server
        .filter_items(Parameters(FilterItemsParams {
            include_archived: false,
            any_of: vec![],
            linked_from: None,
            filters: vec![],
            types: Some(vec!["Ticket".to_string()]),
            type_mode: None,
            under: None,
            sort: None,
            limit: Some(2),
            offset: Some(2),
            detail: Detail::Meta,
            fields: None,
        }))
        .await
        .unwrap();
    let value: Value = serde_json::from_str(&text_of(result)).unwrap();
    assert_eq!(value["total"], 3);
    assert!(value.get("next_offset").is_none());
}

#[tokio::test]
async fn browse_items_root_with_type_uses_server_side_query() {
    let (server, mock) = make_server().await;
    Mock::given(method("POST"))
        .and(path("/item/query"))
        .and(body_json(json!({
            "include_archived": false, "types": ["Book"], "type_mode": "", "under": null, "filters": [],
            "sort": {"key": "title", "direction": ""}, "limit": 2, "offset": 4
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "items": [item(8, "B", "")], "total": 5
        })))
        .expect(1)
        .mount(&mock)
        .await;

    let result = server
        .browse_items(Parameters(BrowseItemsParams {
            include_archived: false,
            mode: BrowseMode::Root,
            type_filter: Some("Book".to_string()),
            sort: Some(SortParam {
                key: "title".to_string(),
                direction: None,
            }),
            limit: Some(2),
            offset: Some(4),
            detail: Detail::Meta,
            fields: None,
        }))
        .await
        .unwrap();
    let value: Value = serde_json::from_str(&text_of(result)).unwrap();
    assert_eq!(value["total"], 5);
}

#[tokio::test]
async fn filter_items_fields_projects_named_attributes_only() {
    let (server, mock) = make_server().await;
    let mut row = typed_item(3, "Z001 Thing", &"long body ".repeat(50));
    row["attributes"] = json!({"Status": "Working", "Priority": 5, "TicketID": "Z001"});
    Mock::given(method("POST"))
        .and(path("/item/query"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"items": [row], "total": 1})))
        .mount(&mock)
        .await;

    let result = server
        .filter_items(Parameters(FilterItemsParams {
            include_archived: false,
            any_of: vec![],
            linked_from: None,
            filters: vec![],
            types: Some(vec!["Ticket".to_string()]),
            type_mode: None,
            under: None,
            sort: None,
            limit: None,
            offset: None,
            detail: Detail::Full,
            fields: Some(vec!["TicketID".to_string(), "Status".to_string()]),
        }))
        .await
        .unwrap();
    let value: Value = serde_json::from_str(&text_of(result)).unwrap();
    assert_eq!(
        value["items"][0],
        json!({
            "archived": false, "item_id": 3, "title": "Z001 Thing", "types": ["Ticket"],
            "attributes": {"TicketID": "Z001", "Status": "Working"}
        })
    );
}

#[tokio::test]
async fn search_items_fields_keeps_match_info() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/item/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([{
            "item_id": 3, "title": "Z001", "content": "body", "attributes": {"Status": "To Do"},
            "types": [], "links": [], "match_scope": "title", "snippet": "Z001"
        }])))
        .mount(&mock)
        .await;

    let result = server
        .search_items(Parameters(SearchItemsParams {
            include_archived: false,
            term: "Z001".to_string(),
            limit: None,
            offset: None,
            scope: None,
            regex: None,
            detail: Detail::Summary,
            fields: Some(vec!["Status".to_string()]),
        }))
        .await
        .unwrap();
    let value: Value = serde_json::from_str(&text_of(result)).unwrap();
    assert_eq!(value["items"][0]["attributes"], json!({"Status": "To Do"}));
    assert_eq!(value["items"][0]["match_scope"], "title");
    assert!(value["items"][0].get("preview").is_none());
}

async fn mount_status_item(mock: &MockServer, id: i64, title: &str) {
    Mock::given(method("GET"))
        .and(path(format!("/item/id/{id}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(item(id, title, "")))
        .mount(mock)
        .await;
}

#[tokio::test]
async fn set_status_updates_items_and_comments_each_in_one_call() {
    let (server, mock) = make_server().await;
    mount_status_item(&mock, 3, "Z001").await;
    mount_status_item(&mock, 4, "Z002").await;
    Mock::given(method("PATCH"))
        .and(path_regex(r"^/item/[34]/attr$"))
        .and(body_json(json!({"Status": "Rejected"})))
        .respond_with(ResponseTemplate::new(200))
        .expect(2)
        .mount(&mock)
        .await;
    Mock::given(method("POST"))
        .and(path("/comment"))
        .respond_with(ResponseTemplate::new(200).set_body_json(comment(21, 3, "Z001")))
        .expect(2)
        .mount(&mock)
        .await;

    let result = server
        .set_status(Parameters(SetStatusParams {
            items: vec!["#3".to_string(), "#4".to_string()],
            status: "Rejected".to_string(),
            comment: Some("Superseded by sync v2".to_string()),
        }))
        .await
        .unwrap();
    let value: Value = serde_json::from_str(&text_of(result)).unwrap();
    assert_eq!(value["updated"], 2);
    assert_eq!(value["failed"], 0);
    assert_eq!(value["items"][0]["status"], "Rejected");
    assert_eq!(value["items"][0]["comment_id"], 21);
}

#[tokio::test]
async fn set_status_requires_a_comment_for_hold_blocked_and_rejected() {
    let (server, _mock) = make_server().await;
    for status in ["Hold", "blocked", "Rejected"] {
        for comment in [None, Some("  ".to_string())] {
            let err = server
                .set_status(Parameters(SetStatusParams {
                    items: vec!["#3".to_string()],
                    status: status.to_string(),
                    comment,
                }))
                .await
                .unwrap_err();
            assert_eq!(err.code, ErrorCode::INVALID_PARAMS, "{status}");
        }
    }
    let err = server
        .set_status(Parameters(SetStatusParams {
            items: vec![],
            status: "Working".to_string(),
            comment: None,
        }))
        .await
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::INVALID_PARAMS);
}

#[tokio::test]
async fn set_status_reports_per_item_failures_and_continues() {
    let (server, mock) = make_server().await;
    mount_status_item(&mock, 4, "Z002").await;
    Mock::given(method("GET"))
        .and(path("/item/id/3"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock)
        .await;
    Mock::given(method("PATCH"))
        .and(path("/item/4/attr"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&mock)
        .await;

    let result = server
        .set_status(Parameters(SetStatusParams {
            items: vec!["#3".to_string(), "#4".to_string()],
            status: "Working".to_string(),
            comment: None,
        }))
        .await
        .unwrap();
    let value: Value = serde_json::from_str(&text_of(result)).unwrap();
    assert_eq!(value["updated"], 1);
    assert_eq!(value["failed"], 1);
    assert!(value["items"][0]["error"].is_string());
    assert_eq!(value["items"][1]["title"], "Z002");
}

#[tokio::test]
async fn wiki_stats_omits_per_type_unless_requested() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/item_type/summary"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {"type_id": 1, "is_system": false, "name": "Ticket",
             "required_attributes_count": 1, "item_count": 4}
        ])))
        .mount(&mock)
        .await;
    Mock::given(method("GET"))
        .and(path("/attribute"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&mock)
        .await;
    Mock::given(method("GET"))
        .and(path("/item/recent"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&mock)
        .await;

    let value: Value = serde_json::from_str(&text_of(
        server
            .wiki_stats(Parameters(WikiStatsParams::default()))
            .await
            .unwrap(),
    ))
    .unwrap();
    assert_eq!(value["totals"]["type_assignment_count"], 4);
    assert!(value.get("per_type").is_none());

    let value: Value = serde_json::from_str(&text_of(
        server
            .wiki_stats(Parameters(WikiStatsParams {
                include_types: Some(true),
            }))
            .await
            .unwrap(),
    ))
    .unwrap();
    assert_eq!(value["per_type"][0]["name"], "Ticket");
}

fn kind(id: i64, key: &str, base_type: &str, config: Value) -> Value {
    json!({"kind_id": id, "key": key, "description": "", "is_system": false,
           "base_type": base_type, "config": config})
}

async fn mount_kinds_and_ticket_type(mock: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/attribute"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            kind(
                1,
                "Status",
                "dropdown",
                json!({"values": ["To Do", "Working"]})
            ),
            kind(2, "Priority", "integer", json!({"min": 1, "max": 10})),
            kind(3, "Date", "date", json!({})),
        ])))
        .mount(mock)
        .await;
    Mock::given(method("GET"))
        .and(path("/item_type/name/Ticket"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "type_id": 9, "is_system": false, "name": "Ticket", "description": "",
            "required_attributes": ["Status", "Priority"]
        })))
        .mount(mock)
        .await;
}

#[tokio::test]
async fn list_attribute_kinds_filters_by_keys_and_item_type() {
    let (server, mock) = make_server().await;
    mount_kinds_and_ticket_type(&mock).await;
    let keys_of = |result: rmcp::model::CallToolResult| -> Vec<String> {
        let value: Value = serde_json::from_str(&text_of(result)).unwrap();
        value
            .as_array()
            .unwrap()
            .iter()
            .map(|k| k["key"].as_str().unwrap().to_string())
            .collect()
    };

    let all = server
        .list_attribute_kinds(Parameters(ListAttributeKindsParams::default()))
        .await
        .unwrap();
    assert_eq!(keys_of(all), ["Status", "Priority", "Date"]);

    let by_type = server
        .list_attribute_kinds(Parameters(ListAttributeKindsParams {
            keys: None,
            item_type: Some("Ticket".to_string()),
        }))
        .await
        .unwrap();
    assert_eq!(keys_of(by_type), ["Status", "Priority"]);

    let by_keys = server
        .list_attribute_kinds(Parameters(ListAttributeKindsParams {
            keys: Some(vec!["Date".to_string(), "Priority".to_string()]),
            item_type: Some("Ticket".to_string()),
        }))
        .await
        .unwrap();
    assert_eq!(keys_of(by_keys), ["Priority"]);
}

#[tokio::test]
async fn delete_system_attribute_kind_reports_read_only() {
    let (server, mock) = make_server().await;
    Mock::given(method("DELETE"))
        .and(path("/attribute/key/Parent"))
        .and(query_param("force", "true"))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_string("System attribute kind 'Parent' is read-only."),
        )
        .mount(&mock)
        .await;

    let err = server
        .delete_attribute_kind(Parameters(DeleteAttributeKindParams {
            key: "Parent".to_string(),
            force: Some(true),
        }))
        .await
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::INVALID_PARAMS);
    assert!(err.message.contains("read-only"));
}

#[tokio::test]
async fn get_item_type_can_include_required_attribute_kinds() {
    let (server, mock) = make_server().await;
    mount_kinds_and_ticket_type(&mock).await;

    let plain: Value = serde_json::from_str(&text_of(
        server
            .get_item_type(Parameters(ItemTypeNameParam {
                name: "Ticket".to_string(),
                include_attributes: None,
            }))
            .await
            .unwrap(),
    ))
    .unwrap();
    assert!(plain.get("attributes").is_none());

    let rich: Value = serde_json::from_str(&text_of(
        server
            .get_item_type(Parameters(ItemTypeNameParam {
                name: "Ticket".to_string(),
                include_attributes: Some(true),
            }))
            .await
            .unwrap(),
    ))
    .unwrap();
    assert_eq!(rich["required_attributes"], json!(["Status", "Priority"]));
    assert_eq!(rich["attributes"][0]["base_type"], "dropdown");
    assert_eq!(
        rich["attributes"][0]["config"]["values"],
        json!(["To Do", "Working"])
    );
    assert_eq!(rich["attributes"][1]["config"]["max"], 10);
}

#[tokio::test]
async fn archive_action_forwards_reversible_subtree_request() {
    let (server, mock) = make_server().await;
    for archived in [true, false] {
        Mock::given(method("POST"))
            .and(path("/item/42/archive"))
            .and(body_json(json!({"archived": archived, "subtree": true})))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"item_ids": [42,43], "archived": archived})),
            )
            .expect(1)
            .mount(&mock)
            .await;
        let result = server
            .set_archived(Parameters(zealot_mcp::tools::wiki::ArchiveItemParams {
                item: "#42".into(),
                archived,
                subtree: true,
            }))
            .await
            .unwrap();
        let value: Value = serde_json::from_str(&text_of(result)).unwrap();
        assert_eq!(value["archived"], archived);
        assert_eq!(value["item_ids"], json!([42, 43]));
    }
}

#[tokio::test]
async fn archived_linked_items_opt_in_is_forwarded() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/item/id/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(item(42, "Root", "")))
        .mount(&mock)
        .await;
    Mock::given(method("GET")).and(path("/item/children/42")).and(query_param("include_archived", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([{"item_id":43,"title":"Child","content":"", "attributes":{"Archived":true},"types":[],"links":[],"archived":true}])))
        .expect(1).mount(&mock).await;
    let result = server
        .get_linked_items(Parameters(LinkedItemsParams {
            item: "#42".into(),
            direction: LinkDirection::Children,
            include_archived: true,
            detail: Detail::Meta,
            fields: None,
            limit: None,
            offset: None,
        }))
        .await
        .unwrap();
    assert!(text_of(result).contains("\"archived\":true"));
}

#[tokio::test]
async fn filter_items_sends_any_of_and_resolved_linked_from() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/item/title/October%202026"))
        .respond_with(ResponseTemplate::new(200).set_body_json(item(4753, "October 2026", "")))
        .mount(&mock)
        .await;
    Mock::given(method("POST"))
        .and(path("/item/query"))
        .and(body_json(json!({
            "types": [], "include_archived": false, "type_mode": "", "under": null,
            "filters": [{"key": "Date", "op": "missing", "value": null, "list_mode": "any"}],
            "any_of": [[{"key": "Week", "op": "eq", "value": "$this_week", "list_mode": "any"}]],
            "linked_from": 4753, "sort": null, "limit": 50, "offset": 0
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"items": [], "total": 0})))
        .expect(1)
        .mount(&mock)
        .await;

    let filter = |key: &str, op: &str, value: Value| AttributeFilterParam {
        key: key.to_string(),
        op: op.to_string(),
        value,
        list_mode: None,
    };
    server
        .filter_items(Parameters(FilterItemsParams {
            include_archived: false,
            filters: vec![filter("Date", "missing", Value::Null)],
            any_of: vec![vec![filter("Week", "eq", json!("$this_week"))]],
            linked_from: Some("October 2026".to_string()),
            types: None,
            type_mode: None,
            under: None,
            sort: None,
            limit: None,
            offset: None,
            detail: Detail::Meta,
            fields: None,
        }))
        .await
        .unwrap();
}

#[tokio::test]
async fn run_view_resolves_title_and_shapes_items_by_view_columns() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/item/title/AI%20Queue"))
        .respond_with(ResponseTemplate::new(200).set_body_json(item(10, "AI Queue", "")))
        .mount(&mock)
        .await;
    let mut plan = item(3, "Plan A", "long content that should not appear");
    plan["attributes"] = json!({"Status": "To Do", "Executor": "LLM"});
    Mock::given(method("POST"))
        .and(path("/view/10/run"))
        .and(body_json(json!({"limit": null, "offset": 2})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "view_id": 10, "title": "AI Queue", "total": 5, "offset": 2,
            "items": [plan],
            "group_by": "Parent",
            "groups": [{"key": 1, "label": "Zealot", "item_ids": [3]}],
            "columns": ["Status"]
        })))
        .expect(1)
        .mount(&mock)
        .await;

    let result = server
        .run_view(Parameters(RunViewParams {
            view: "AI Queue".to_string(),
            limit: None,
            offset: Some(2),
            detail: None,
            fields: None,
        }))
        .await
        .unwrap();
    let value: Value = serde_json::from_str(&text_of(result)).unwrap();
    assert_eq!(value["total"], 5);
    assert_eq!(value["next_offset"], 3);
    assert_eq!(value["groups"][0]["label"], "Zealot");
    // Default shape comes from the view's columns: Status only, no content.
    assert_eq!(value["items"][0]["attributes"], json!({"Status": "To Do"}));
    assert!(value["items"][0].get("content").is_none());
}

#[tokio::test]
async fn validate_view_posts_the_definition() {
    let (server, mock) = make_server().await;
    let definition = json!({"query": {"types": ["Plan"]}});
    Mock::given(method("POST"))
        .and(path("/view/validate"))
        .and(body_json(json!({"definition": definition})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "definition": definition, "resolved_query": {"types": ["Plan"]},
            "today": "2026-10-02", "time_zone": "Europe/London", "total": 4, "elapsed_ms": 1
        })))
        .expect(1)
        .mount(&mock)
        .await;

    let result = server
        .validate_view(Parameters(ValidateViewParams { definition }))
        .await
        .unwrap();
    let value: Value = serde_json::from_str(&text_of(result)).unwrap();
    assert_eq!(value["total"], 4);
}

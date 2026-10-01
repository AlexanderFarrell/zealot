//! Targeted content tools (Z194): content `version` on reads, section and line
//! reads, exact-text edits, section rewrites, and line-level grep.

use rmcp::handler::server::wrapper::Parameters;
use serde_json::{Value, json};
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
use zealot_domain::item_content::content_version;
use zealot_mcp::{
    client::ZealotClient,
    output::Detail,
    tools::{
        ZealotServer,
        wiki::{
            AppendToItemParams, EditItemParams, GrepItemsParams, ItemOutlineParams, ItemRefParam,
            ReadItemParams, ReplaceSectionParams, UpdateItemParams,
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

fn text_of(result: rmcp::model::CallToolResult) -> String {
    result.content[0].as_text().unwrap().text.clone()
}

fn parse(result: rmcp::model::CallToolResult) -> Value {
    serde_json::from_str(&text_of(result)).unwrap()
}

/// An item as an older server sends it: no `version` field.
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

async fn mount_item(mock: &MockServer, body: Value) {
    let id = body["item_id"].as_i64().unwrap();
    Mock::given(method("GET"))
        .and(path(format!("/item/id/{id}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(mock)
        .await;
}

const DOC: &str = "# Goal\nShip it.\n\n## Detail\nMore.\n# Notes\nEnd.";

#[tokio::test]
async fn get_item_returns_version_at_every_detail_level() {
    let (server, mock) = make_server().await;
    mount_item(&mock, item(7, "Doc", DOC)).await;

    for detail in [Detail::Meta, Detail::Summary, Detail::Full] {
        let value = parse(
            server
                .get_item(Parameters(ItemRefParam {
                    item: "7".to_string(),
                    detail,
                }))
                .await
                .unwrap(),
        );
        assert_eq!(value["version"], content_version(DOC), "{detail:?}");
    }
}

#[tokio::test]
async fn get_item_prefers_the_server_version() {
    let (server, mock) = make_server().await;
    let mut body = item(7, "Doc", DOC);
    body["version"] = json!("server-version");
    mount_item(&mock, body).await;

    let value = parse(
        server
            .get_item(Parameters(ItemRefParam {
                item: "7".to_string(),
                detail: Detail::Full,
            }))
            .await
            .unwrap(),
    );
    assert_eq!(value["version"], "server-version");
}

#[tokio::test]
async fn get_item_outline_returns_version() {
    let (server, mock) = make_server().await;
    mount_item(&mock, item(7, "Doc", DOC)).await;

    let value = parse(
        server
            .get_item_outline(Parameters(ItemOutlineParams {
                item: "7".to_string(),
            }))
            .await
            .unwrap(),
    );
    assert_eq!(value["version"], content_version(DOC));
    assert_eq!(value["headings"].as_array().unwrap().len(), 3);
}

fn read(item: &str, section: Option<&str>, lines: Option<&str>) -> Parameters<ReadItemParams> {
    Parameters(ReadItemParams {
        item: item.to_string(),
        section: section.map(str::to_string),
        lines: lines.map(str::to_string),
    })
}

#[tokio::test]
async fn read_item_returns_one_section_with_line_numbers() {
    let (server, mock) = make_server().await;
    mount_item(&mock, item(7, "Doc", DOC)).await;

    let value = parse(
        server
            .read_item(read("7", Some("Goal"), None))
            .await
            .unwrap(),
    );
    assert_eq!(value["version"], content_version(DOC));
    assert_eq!(value["start_line"], 1);
    assert_eq!(value["end_line"], 5);
    assert_eq!(value["total_lines"], 7);
    assert_eq!(value["heading"], json!({"level": 1, "text": "Goal"}));
    assert_eq!(
        value["content"],
        "1\t# Goal\n2\tShip it.\n3\t\n4\t## Detail\n5\tMore."
    );

    // The last section runs to the end of the body.
    let value = parse(
        server
            .read_item(read("7", Some("# Notes"), None))
            .await
            .unwrap(),
    );
    assert_eq!(value["content"], "6\t# Notes\n7\tEnd.");
}

#[tokio::test]
async fn read_item_reads_a_line_range() {
    let (server, mock) = make_server().await;
    mount_item(&mock, item(7, "Doc", DOC)).await;

    let value = parse(
        server
            .read_item(read("7", None, Some("4-5")))
            .await
            .unwrap(),
    );
    assert_eq!(value["content"], "4\t## Detail\n5\tMore.");
    assert!(value.get("heading").is_none());
}

#[tokio::test]
async fn read_item_errors_name_the_reason() {
    let (server, mock) = make_server().await;
    mount_item(&mock, item(7, "Doc", "# A\n## Tasks\nx\n# B\n## Tasks\ny")).await;

    let missing = server
        .read_item(read("7", Some("Nope"), None))
        .await
        .unwrap_err();
    assert!(missing.message.contains("not found"), "{}", missing.message);
    assert!(missing.message.contains("## Tasks"), "{}", missing.message);

    let ambiguous = server
        .read_item(read("7", Some("Tasks"), None))
        .await
        .unwrap_err();
    assert!(
        ambiguous.message.contains("[2, 5]"),
        "{}",
        ambiguous.message
    );

    let range = server
        .read_item(read("7", None, Some("40")))
        .await
        .unwrap_err();
    assert!(range.message.contains("6 lines"), "{}", range.message);

    let both = server
        .read_item(read("7", Some("A"), Some("1")))
        .await
        .unwrap_err();
    assert!(both.message.contains("not both"));
}

fn write_reply(id: i64) -> Value {
    json!({
        "item_id": id,
        "title": "Doc",
        "version": "v2",
        "content_chars": 40,
        "changed_lines": {"start": 2, "removed": 1, "added": 1}
    })
}

#[tokio::test]
async fn edit_item_sends_one_small_request_and_returns_a_compact_reply() {
    let (server, mock) = make_server().await;
    Mock::given(method("POST"))
        .and(path("/item/7/content"))
        .and(body_json(json!({
            "op": "replace",
            "old": "Ship it.",
            "new": "Shipped.",
            "replace_all": false,
            "expected_version": "v1"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(write_reply(7)))
        .expect(1)
        .mount(&mock)
        .await;

    let reply = text_of(
        server
            .edit_item(Parameters(EditItemParams {
                item: "#7".to_string(),
                old: "Ship it.".to_string(),
                new: "Shipped.".to_string(),
                replace_all: None,
                expected_version: Some("v1".to_string()),
            }))
            .await
            .unwrap(),
    );
    let value: Value = serde_json::from_str(&reply).unwrap();
    assert_eq!(value, write_reply(7));
    assert!(reply.len() < 1024);
}

#[tokio::test]
async fn edit_item_by_title_resolves_the_id_first() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/item/title/Doc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(item(7, "Doc", DOC)))
        .expect(1)
        .mount(&mock)
        .await;
    Mock::given(method("POST"))
        .and(path("/item/7/content"))
        .respond_with(ResponseTemplate::new(200).set_body_json(write_reply(7)))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .edit_item(Parameters(EditItemParams {
            item: "Doc".to_string(),
            old: "a".to_string(),
            new: "b".to_string(),
            replace_all: Some(true),
            expected_version: None,
        }))
        .await
        .unwrap();
}

#[tokio::test]
async fn edit_item_surfaces_conflicts_and_bad_matches() {
    let (server, mock) = make_server().await;
    Mock::given(method("POST"))
        .and(path("/item/7/content"))
        .and(body_json(json!({"op": "replace", "old": "x", "new": "y", "replace_all": false, "expected_version": "old"})))
        .respond_with(ResponseTemplate::new(409).set_body_string(
            "the item changed since version old (now v2); re-read it and retry the edit",
        ))
        .mount(&mock)
        .await;
    Mock::given(method("POST"))
        .and(path("/item/7/content"))
        .and(body_json(json!({"op": "replace", "old": "TODO", "new": "y", "replace_all": false})))
        .respond_with(ResponseTemplate::new(400).set_body_string(
            "old text matches 2 times (lines [8, 9]); add surrounding text to make it unique, or set replace_all",
        ))
        .mount(&mock)
        .await;

    let edit = |old: &str, version: Option<&str>| {
        Parameters(EditItemParams {
            item: "7".to_string(),
            old: old.to_string(),
            new: "y".to_string(),
            replace_all: None,
            expected_version: version.map(str::to_string),
        })
    };
    let conflict = server.edit_item(edit("x", Some("old"))).await.unwrap_err();
    assert!(conflict.message.contains("409"), "{}", conflict.message);
    assert!(conflict.message.contains("re-read"), "{}", conflict.message);

    let ambiguous = server.edit_item(edit("TODO", None)).await.unwrap_err();
    assert!(
        ambiguous.message.contains("matches 2 times"),
        "{}",
        ambiguous.message
    );
}

#[tokio::test]
async fn append_to_item_forwards_expected_version() {
    let (server, mock) = make_server().await;
    Mock::given(method("POST"))
        .and(path("/item/7/content"))
        .and(body_json(
            json!({"op": "append", "text": "- log", "expected_version": "v1"}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(write_reply(7)))
        .expect(1)
        .mount(&mock)
        .await;

    server
        .append_to_item(Parameters(AppendToItemParams {
            item: "7".to_string(),
            text: "- log".to_string(),
            expected_version: Some("v1".to_string()),
        }))
        .await
        .unwrap();
}

#[tokio::test]
async fn update_item_forwards_expected_version_and_reports_changed_lines() {
    let (server, mock) = make_server().await;
    mount_item(&mock, item(7, "Doc", "a\nb\nc")).await;
    Mock::given(method("PATCH"))
        .and(path("/item/7"))
        .and(body_json(
            json!({"item_id": 7, "content": "a\nB\nc", "expected_version": "v1"}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(item(7, "Doc", "a\nB\nc")))
        .expect(1)
        .mount(&mock)
        .await;

    let value = parse(
        server
            .update_item(Parameters(UpdateItemParams {
                item: "7".to_string(),
                title: None,
                content: Some("a\nB\nc".to_string()),
                expected_version: Some("v1".to_string()),
                full: None,
            }))
            .await
            .unwrap(),
    );
    assert_eq!(
        value,
        json!({
            "item_id": 7,
            "title": "Doc",
            "version": content_version("a\nB\nc"),
            "content_chars": 5,
            "changed_lines": {"start": 2, "removed": 1, "added": 1}
        })
    );
}

#[tokio::test]
async fn replace_section_posts_heading_and_body() {
    let (server, mock) = make_server().await;
    Mock::given(method("POST"))
        .and(path("/item/7/content"))
        .and(body_json(json!({
            "op": "replace_section",
            "heading": "## Detail",
            "content": "Rewritten.",
            "expected_version": "v1"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(write_reply(7)))
        .expect(1)
        .mount(&mock)
        .await;

    let value = parse(
        server
            .replace_section(Parameters(ReplaceSectionParams {
                item: "7".to_string(),
                heading: "## Detail".to_string(),
                content: "Rewritten.".to_string(),
                expected_version: Some("v1".to_string()),
            }))
            .await
            .unwrap(),
    );
    assert_eq!(value["version"], "v2");
}

#[tokio::test]
async fn replace_section_reports_an_ambiguous_heading() {
    let (server, mock) = make_server().await;
    Mock::given(method("POST"))
        .and(path("/item/7/content"))
        .respond_with(ResponseTemplate::new(400).set_body_string(
            "heading 'Tasks' is ambiguous: it appears on lines [2, 5]; include its level (for example '## Tasks') or read by line range",
        ))
        .mount(&mock)
        .await;

    let err = server
        .replace_section(Parameters(ReplaceSectionParams {
            item: "7".to_string(),
            heading: "Tasks".to_string(),
            content: "x".to_string(),
            expected_version: None,
        }))
        .await
        .unwrap_err();
    assert!(err.message.contains("ambiguous"), "{}", err.message);
}

#[tokio::test]
async fn grep_items_resolves_under_and_returns_line_hits() {
    let (server, mock) = make_server().await;
    Mock::given(method("GET"))
        .and(path("/item/title/Zealot"))
        .respond_with(ResponseTemplate::new(200).set_body_json(item(11, "Zealot", "")))
        .expect(1)
        .mount(&mock)
        .await;
    let page = json!({
        "count": 1,
        "items": [{"item_id": 12, "title": "Ticket A", "match_count": 1, "lines": ["3:fixes Z190."]}]
    });
    Mock::given(method("POST"))
        .and(path("/item/grep"))
        .and(body_json(json!({
            "pattern": "Z190",
            "include_archived": false,
            "regex": false,
            "under": 11,
            "types": ["Ticket"],
            "context_lines": 0,
            "limit": 20,
            "offset": 0
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(page.clone()))
        .expect(1)
        .mount(&mock)
        .await;

    let value = parse(
        server
            .grep_items(Parameters(GrepItemsParams {
                include_archived: false,
                pattern: "Z190".to_string(),
                regex: None,
                under: Some("Zealot".to_string()),
                types: Some(vec!["Ticket".to_string()]),
                context_lines: None,
                limit: None,
                offset: None,
            }))
            .await
            .unwrap(),
    );
    assert_eq!(value, page);
}

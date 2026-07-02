use serde_json::json;
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};
use zealot_client::types::MediaEntry;
use zealot_client::{ApiError, ZealotClient};
use zealot_domain::item::SearchScope;

async fn setup() -> (MockServer, ZealotClient) {
    let server = MockServer::start().await;
    let client = ZealotClient::new(server.uri(), "test-key");
    (server, client)
}

fn item_json(id: i64, title: &str) -> serde_json::Value {
    json!({
        "item_id": id,
        "title": title,
        "content": "hello",
        "attributes": {},
        "types": [],
        "links": []
    })
}

#[tokio::test]
async fn sends_api_key_header() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/item/id/1"))
        .and(header("X-API-Key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(item_json(1, "Note")))
        .expect(1)
        .mount(&server)
        .await;

    let item = client.get_item(1).await.unwrap();
    assert_eq!(item.title, "Note");
}

#[tokio::test]
async fn title_lookup_is_url_encoded() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/item/title/My%20Note"))
        .respond_with(ResponseTemplate::new(200).set_body_json(item_json(2, "My Note")))
        .mount(&server)
        .await;

    let item = client.get_item_by_title("My Note").await.unwrap();
    assert_eq!(item.item_id, 2);
}

#[tokio::test]
async fn search_passes_scope_and_flags() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/item/search"))
        .and(query_param("term", "recipe"))
        .and(query_param("scope", "content"))
        .and(query_param("regex", "true"))
        .and(query_param("limit", "5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([{
            "item_id": 3,
            "title": "Recipes",
            "content": "",
            "attributes": {},
            "types": [],
            "links": [],
            "match_scope": "content",
            "snippet": "…a recipe for…"
        }])))
        .mount(&server)
        .await;

    let results = client
        .search_items("recipe", SearchScope::Content, true, 5, 0)
        .await
        .unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].snippet.as_deref(), Some("…a recipe for…"));
}

#[tokio::test]
async fn not_found_maps_to_api_error() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/item/id/99"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    assert!(matches!(client.get_item(99).await, Err(ApiError::NotFound)));
}

#[tokio::test]
async fn unauthorized_is_detectable() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/auth/is_logged_in"))
        .respond_with(ResponseTemplate::new(401).set_body_string("unauthorized"))
        .mount(&server)
        .await;

    let err = client.whoami().await.unwrap_err();
    assert!(err.is_unauthorized());
}

#[tokio::test]
async fn create_api_key_posts_credentials() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/auth/api_key"))
        .and(body_json(json!({
            "username": "alex",
            "password": "pw",
            "label": "CLI (test)"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "key": "zk_raw",
            "api_key_id": 12,
            "label": "CLI (test)",
            "created_at": "2026-07-02 10:00:00"
        })))
        .mount(&server)
        .await;

    let resp =
        zealot_client::api::auth::create_api_key_with_credentials(&server.uri(), "alex", "pw", "CLI (test)")
            .await
            .unwrap();
    assert_eq!(resp.key, "zk_raw");
    assert_eq!(resp.api_key_id, 12);
}

#[tokio::test]
async fn repeat_status_put_sends_dto() {
    let (server, client) = setup().await;
    Mock::given(method("PUT"))
        .and(path("/repeat/status"))
        .and(body_json(json!({
            "item_id": 4,
            "date": "2026-07-02",
            "status": "Complete",
            "comment": null
        })))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let dto = zealot_domain::repeat::UpdateRepeatEntryDto {
        item_id: 4,
        date: "2026-07-02".into(),
        status: Some("Complete".into()),
        comment: None,
    };
    client.set_repeat_status(&dto).await.unwrap();
}

#[tokio::test]
async fn media_get_distinguishes_directories_from_files() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/media"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "files": [{"path": "docs", "size": 0, "is_folder": true, "modified_at": 1723198514}]
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/media/docs/a.txt"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string("hello")
                .insert_header("content-type", "text/plain"),
        )
        .mount(&server)
        .await;

    match client.media_get("").await.unwrap() {
        MediaEntry::Directory(files) => assert_eq!(files[0].path, "docs"),
        MediaEntry::File(_) => panic!("expected directory"),
    }
    match client.media_get("docs/a.txt").await.unwrap() {
        MediaEntry::File(download) => assert_eq!(download.bytes, b"hello"),
        MediaEntry::Directory(_) => panic!("expected file"),
    }
}

#[tokio::test]
async fn export_captures_disposition_filename() {
    let (server, client) = setup().await;
    Mock::given(method("GET"))
        .and(path("/item/id/1/export/pdf"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(b"%PDF".to_vec())
                .insert_header("content-type", "application/pdf")
                .insert_header("content-disposition", "attachment; filename=\"Note.pdf\""),
        )
        .mount(&server)
        .await;

    let download = client.export_item_pdf(1).await.unwrap();
    assert_eq!(download.filename.as_deref(), Some("Note.pdf"));
    assert_eq!(download.bytes, b"%PDF");
}

use reqwest::StatusCode;
use serde_json::json;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};
use zealot_mcp::client::{ApiError, ZealotClient};

fn make_client(server: &MockServer) -> ZealotClient {
    ZealotClient::new(server.uri(), "test-api-key")
}

// ── Constructor ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn client_strips_trailing_slash() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/item"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .expect(1)
        .mount(&server)
        .await;

    let url_with_slash = format!("{}/", server.uri());
    let client = ZealotClient::new(url_with_slash, "key");
    let _: serde_json::Value = client.get("/item").await.unwrap();
}

// ── get() ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn get_success_returns_deserialized_body() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/item/id/42"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": 42, "title": "Test"})))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let result: serde_json::Value = client.get("/item/id/42").await.unwrap();
    assert_eq!(result["id"], 42);
    assert_eq!(result["title"], "Test");
}

#[tokio::test]
async fn get_sends_api_key_header() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/item"))
        .and(header("X-API-Key", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .expect(1)
        .mount(&server)
        .await;

    let client = make_client(&server);
    let _: serde_json::Value = client.get("/item").await.unwrap();
}

#[tokio::test]
async fn get_returns_not_found_on_404() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/item/id/999"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let result: Result<serde_json::Value, _> = client.get("/item/id/999").await;
    assert!(matches!(result, Err(ApiError::NotFound)));
}

#[tokio::test]
async fn get_returns_http_error_on_500() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/item"))
        .respond_with(ResponseTemplate::new(500).set_body_string("internal server error"))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let result: Result<serde_json::Value, _> = client.get("/item").await;
    match result {
        Err(ApiError::Http { status, message }) => {
            assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
            assert!(message.contains("internal server error"));
        }
        other => panic!("expected Http error, got {other:?}"),
    }
}

#[tokio::test]
async fn get_returns_http_error_when_json_invalid() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/item"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json at all"))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let result: Result<serde_json::Value, _> = client.get("/item").await;
    match result {
        Err(ApiError::Http { message, .. }) => {
            assert!(
                message.contains("JSON decode failed"),
                "message was: {message}"
            );
        }
        other => panic!("expected Http error, got {other:?}"),
    }
}

// ── post() ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn post_success_sends_json_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/item"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": 1})))
        .expect(1)
        .mount(&server)
        .await;

    let client = make_client(&server);
    let body = json!({"title": "New", "content": "body"});
    let result: serde_json::Value = client.post("/item", &body).await.unwrap();
    assert_eq!(result["id"], 1);
}

#[tokio::test]
async fn post_returns_not_found_on_404() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/item"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let result: Result<serde_json::Value, _> = client.post("/item", &json!({})).await;
    assert!(matches!(result, Err(ApiError::NotFound)));
}

#[tokio::test]
async fn post_returns_http_error_on_422() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/item"))
        .respond_with(ResponseTemplate::new(422).set_body_string("validation failed"))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let result: Result<serde_json::Value, _> = client.post("/item", &json!({})).await;
    match result {
        Err(ApiError::Http { status, .. }) => assert_eq!(status.as_u16(), 422),
        other => panic!("expected Http error, got {other:?}"),
    }
}

// ── patch() ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn patch_success_sends_patch_method() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/item/5"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": 5})))
        .expect(1)
        .mount(&server)
        .await;

    let client = make_client(&server);
    let result: serde_json::Value = client
        .patch("/item/5", &json!({"title": "Updated"}))
        .await
        .unwrap();
    assert_eq!(result["id"], 5);
}

#[tokio::test]
async fn patch_returns_not_found_on_404() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/item/999"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let result: Result<serde_json::Value, _> = client.patch("/item/999", &json!({})).await;
    assert!(matches!(result, Err(ApiError::NotFound)));
}

// ── put() ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn put_success_sends_put_method() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/repeat/status"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
        .expect(1)
        .mount(&server)
        .await;

    let client = make_client(&server);
    let result: serde_json::Value = client.put("/repeat/status", &json!({})).await.unwrap();
    assert_eq!(result["ok"], true);
}

#[tokio::test]
async fn put_returns_not_found_on_404() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/repeat/status"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let client = make_client(&server);
    let result: Result<serde_json::Value, _> = client.put("/repeat/status", &json!({})).await;
    assert!(matches!(result, Err(ApiError::NotFound)));
}

// ── delete() ──────────────────────────────────────────────────────────────────

#[tokio::test]
async fn delete_success_on_200() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/item/3"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = make_client(&server);
    client.delete("/item/3").await.unwrap();
}

#[tokio::test]
async fn delete_success_on_204() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/item/3"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let client = make_client(&server);
    client.delete("/item/3").await.unwrap();
}

#[tokio::test]
async fn delete_returns_not_found_on_404() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/item/999"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let client = make_client(&server);
    assert!(matches!(
        client.delete("/item/999").await,
        Err(ApiError::NotFound)
    ));
}

#[tokio::test]
async fn delete_returns_http_error_on_403() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/item/1"))
        .respond_with(ResponseTemplate::new(403).set_body_string("forbidden"))
        .mount(&server)
        .await;

    let client = make_client(&server);
    match client.delete("/item/1").await {
        Err(ApiError::Http { status, message }) => {
            assert_eq!(status.as_u16(), 403);
            assert!(message.contains("forbidden"));
        }
        other => panic!("expected Http error, got {other:?}"),
    }
}

// ── post_no_response() ────────────────────────────────────────────────────────

#[tokio::test]
async fn post_no_response_success() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/item/7/assign_type/Goal"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = make_client(&server);
    client
        .post_no_response("/item/7/assign_type/Goal", &json!({}))
        .await
        .unwrap();
}

#[tokio::test]
async fn post_no_response_not_found() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/item/7/assign_type/Goal"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let client = make_client(&server);
    assert!(matches!(
        client
            .post_no_response("/item/7/assign_type/Goal", &json!({}))
            .await,
        Err(ApiError::NotFound)
    ));
}

#[tokio::test]
async fn post_no_response_http_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/item/7/assign_type/Goal"))
        .respond_with(ResponseTemplate::new(500).set_body_string("oops"))
        .mount(&server)
        .await;

    let client = make_client(&server);
    match client
        .post_no_response("/item/7/assign_type/Goal", &json!({}))
        .await
    {
        Err(ApiError::Http { status, .. }) => assert_eq!(status.as_u16(), 500),
        other => panic!("expected Http error, got {other:?}"),
    }
}

// ── put_no_response() ─────────────────────────────────────────────────────────

#[tokio::test]
async fn put_no_response_success() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/repeat/status"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let client = make_client(&server);
    client
        .put_no_response("/repeat/status", &json!({}))
        .await
        .unwrap();
}

#[tokio::test]
async fn put_no_response_not_found() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/repeat/status"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let client = make_client(&server);
    assert!(matches!(
        client.put_no_response("/repeat/status", &json!({})).await,
        Err(ApiError::NotFound)
    ));
}

// ── patch_no_response() ───────────────────────────────────────────────────────

#[tokio::test]
async fn patch_no_response_success() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/item/5/attr"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = make_client(&server);
    client
        .patch_no_response("/item/5/attr", &json!({"priority": "high"}))
        .await
        .unwrap();
}

#[tokio::test]
async fn patch_no_response_not_found() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/item/5/attr"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let client = make_client(&server);
    assert!(matches!(
        client.patch_no_response("/item/5/attr", &json!({})).await,
        Err(ApiError::NotFound)
    ));
}

#[tokio::test]
async fn patch_no_response_http_error() {
    let server = MockServer::start().await;
    Mock::given(method("PATCH"))
        .and(path("/item/5/attr"))
        .respond_with(ResponseTemplate::new(500).set_body_string("error"))
        .mount(&server)
        .await;

    let client = make_client(&server);
    match client.patch_no_response("/item/5/attr", &json!({})).await {
        Err(ApiError::Http { status, .. }) => assert_eq!(status.as_u16(), 500),
        other => panic!("expected Http error, got {other:?}"),
    }
}

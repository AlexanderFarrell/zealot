use std::collections::HashMap;

use axum::{
    Extension, Json, Router,
    body::Body,
    extract::{Path, Query, State},
    http::{StatusCode, header},
    middleware,
    response::Response,
    routing::{delete, get, patch, post},
};
use serde::Deserialize;
use serde_json::Value;
use zealot_app::{
    app::AppState,
    services::item::{ItemServiceError, SearchResult},
};
use zealot_domain::{
    attribute::AttributeFilterDto,
    auth::Actor,
    common::id::Id,
    item::{AddItemDto, Item, ItemDto, SearchResultDto, SearchScope, UpdateItemDto},
};

use crate::http::{
    common::HttpError,
    middleware::{auth_middleware, csrf_middleware},
};

// ─── Embedded fonts for PDF export ───────────────────────────────────────────

static FONT_REGULAR: &[u8] = include_bytes!("../../assets/fonts/LiberationSans-Regular.ttf");
static FONT_BOLD: &[u8] = include_bytes!("../../assets/fonts/LiberationSans-Bold.ttf");
static FONT_ITALIC: &[u8] = include_bytes!("../../assets/fonts/LiberationSans-Italic.ttf");
static FONT_BOLD_ITALIC: &[u8] = include_bytes!("../../assets/fonts/LiberationSans-BoldItalic.ttf");

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(get_root_items).post(add_item))
        .route("/recent", get(get_recent_items))
        .route("/title/{title}", get(get_by_title))
        .route("/id/{item_id}", get(get_by_id))
        .route("/id/{item_id}/export/pdf", get(export_pdf))
        .route("/id/{item_id}/export/docx", get(export_docx))
        .route("/search", get(search_items))
        .route("/children/{item_id}", get(get_children))
        .route("/related/{item_id}", get(get_related))
        .route("/backlinks/{item_id}", get(get_backlinks))
        .route("/filter", post(filter_items))
        .route("/rebuild-links", post(rebuild_links))
        .route("/{item_id}", patch(update_item).delete(delete_item))
        .route("/{item_id}/attr", patch(set_attributes))
        .route("/{item_id}/attr/rename", patch(rename_attribute))
        .route("/{item_id}/attr/{key}", delete(delete_attribute))
        .route(
            "/{item_id}/assign_type/{type_name}",
            post(assign_type).delete(unassign_type),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            csrf_middleware,
        ))
        .route_layer(middleware::map_request_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state)
}

// ─── Auth helper ─────────────────────────────────────────────────────────────

fn require_account(actor: &Actor) -> Result<zealot_domain::account::Account, HttpError> {
    if !actor.is_authenticated() {
        return Err(HttpError::Unauthorized);
    }
    actor.account.clone().ok_or(HttpError::Unauthorized)
}

fn item_service_err(err: ItemServiceError) -> HttpError {
    match err {
        ItemServiceError::NotFound => HttpError::NotFound,
        ItemServiceError::Unauthorized => HttpError::Unauthorized,
        ItemServiceError::Attribute(e) => HttpError::UserError { err: e.to_string() },
        ItemServiceError::InvalidFilter(msg) => HttpError::UserError { err: msg },
        ItemServiceError::InvalidId(msg) => HttpError::UserError { err: msg },
        ItemServiceError::InvalidRegex(msg) => HttpError::UserError { err: msg },
        ItemServiceError::Repo(e) => {
            tracing::error!("Item repo error: {e}");
            HttpError::Internal
        }
    }
}

fn parse_item_id(raw: i64) -> Result<Id, HttpError> {
    Id::try_from(raw).map_err(|e| HttpError::UserError { err: e.to_string() })
}

// ─── Query param structs ──────────────────────────────────────────────────────

#[derive(Deserialize)]
struct RootItemsParams {
    #[serde(rename = "type", default)]
    item_type: Option<String>,
}

const MAX_SEARCH_LIMIT: i64 = 100;
const DEFAULT_FILTER_LIMIT: i64 = 50;
const MAX_FILTER_LIMIT: i64 = 100;

#[derive(Deserialize)]
struct SearchParams {
    #[serde(default)]
    term: String,
    #[serde(default = "default_search_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
    scope: Option<String>,
    #[serde(default)]
    regex: bool,
}

fn default_search_limit() -> i64 {
    20
}

#[derive(Deserialize)]
struct RecentParams {
    #[serde(default = "default_recent_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}

fn default_recent_limit() -> i64 {
    30
}

#[derive(Deserialize)]
struct FilterBody {
    filters: Vec<AttributeFilterDto>,
    #[serde(default = "default_filter_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}

fn default_filter_limit() -> i64 {
    DEFAULT_FILTER_LIMIT
}

#[derive(Deserialize)]
struct RenameAttributeDto {
    old_key: String,
    new_key: String,
}

// ─── Handlers ────────────────────────────────────────────────────────────────

async fn get_root_items(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Query(params): Query<RootItemsParams>,
) -> Result<Json<Vec<ItemDto>>, HttpError> {
    let account = require_account(&actor)?;
    let items = if let Some(type_name) = params.item_type {
        state.services.item.get_items_by_type(&type_name, &account)
    } else {
        state.services.item.get_root_items(&account)
    }
    .map_err(item_service_err)?;
    Ok(Json(items.iter().map(ItemDto::from).collect()))
}

async fn get_recent_items(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Query(params): Query<RecentParams>,
) -> Result<Json<Vec<ItemDto>>, HttpError> {
    let account = require_account(&actor)?;
    let items = state
        .services
        .item
        .get_recent_items(params.limit, params.offset, &account)
        .map_err(item_service_err)?;
    Ok(Json(items.iter().map(ItemDto::from).collect()))
}

async fn get_by_title(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(title): Path<String>,
) -> Result<Json<ItemDto>, HttpError> {
    let account = require_account(&actor)?;
    let items = state
        .services
        .item
        .get_items_by_title(&title, &account)
        .map_err(item_service_err)?;
    items
        .into_iter()
        .next()
        .map(|i| Json(ItemDto::from(&i)))
        .ok_or(HttpError::NotFound)
}

async fn get_by_id(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(item_id): Path<i64>,
) -> Result<Json<ItemDto>, HttpError> {
    let account = require_account(&actor)?;
    let id = parse_item_id(item_id)?;
    match state
        .services
        .item
        .get_item_by_id(&id, &account)
        .map_err(item_service_err)?
    {
        Some(item) => {
            if let Err(e) = state.services.analysis.record_view(&id) {
                tracing::warn!("Failed to record view for item {item_id}: {e}");
            }
            Ok(Json(ItemDto::from(&item)))
        }
        None => Err(HttpError::NotFound),
    }
}

async fn search_items(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<SearchResultDto>>, HttpError> {
    let account = require_account(&actor)?;
    let limit = params.limit.clamp(1, MAX_SEARCH_LIMIT);
    let scope = match params.scope.as_deref() {
        None | Some("title") => SearchScope::Title,
        Some("content") => SearchScope::Content,
        Some("heading") => SearchScope::Heading,
        Some(other) => {
            return Err(HttpError::UserError {
                err: format!("unknown scope: {other}"),
            });
        }
    };
    let results = state
        .services
        .item
        .search_items(
            &params.term,
            scope,
            params.regex,
            limit,
            params.offset,
            &account,
        )
        .map_err(item_service_err)?;
    Ok(Json(
        results.into_iter().map(search_result_to_dto).collect(),
    ))
}

fn search_result_to_dto(r: SearchResult) -> SearchResultDto {
    SearchResultDto {
        item: ItemDto::from(&r.item),
        match_scope: r.match_scope,
        snippet: r.snippet,
    }
}

async fn get_children(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(item_id): Path<i64>,
) -> Result<Json<Vec<ItemDto>>, HttpError> {
    let account = require_account(&actor)?;
    let id = parse_item_id(item_id)?;
    let items = state
        .services
        .item
        .get_children(&id, &account)
        .map_err(item_service_err)?;
    Ok(Json(items.iter().map(ItemDto::from).collect()))
}

async fn get_related(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(item_id): Path<i64>,
) -> Result<Json<Vec<ItemDto>>, HttpError> {
    let account = require_account(&actor)?;
    let id = parse_item_id(item_id)?;
    let items = state
        .services
        .item
        .get_related_items(&id, &account)
        .map_err(item_service_err)?;
    Ok(Json(items.iter().map(ItemDto::from).collect()))
}

async fn get_backlinks(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(item_id): Path<i64>,
) -> Result<Json<Vec<ItemDto>>, HttpError> {
    let account = require_account(&actor)?;
    let id = parse_item_id(item_id)?;
    let items = state
        .services
        .item
        .get_backlinks(&id, &account)
        .map_err(item_service_err)?;
    Ok(Json(items.iter().map(ItemDto::from).collect()))
}

async fn filter_items(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Json(body): Json<FilterBody>,
) -> Result<Json<Vec<ItemDto>>, HttpError> {
    let account = require_account(&actor)?;
    let limit = body.limit.clamp(1, MAX_FILTER_LIMIT);
    let offset = body.offset.max(0);
    let items = state
        .services
        .item
        .filter_items_paginated(&body.filters, limit, offset, &account)
        .map_err(item_service_err)?;
    Ok(Json(items.iter().map(ItemDto::from).collect()))
}

async fn rebuild_links(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
) -> Result<Json<serde_json::Value>, HttpError> {
    let account = require_account(&actor)?;
    let attribute_count = state
        .services
        .item
        .rebuild_links_for_account(&account)
        .map_err(item_service_err)?;
    let wiki_count = state
        .services
        .item
        .rebuild_wiki_links_for_account(&account)
        .map_err(item_service_err)?;
    Ok(Json(
        serde_json::json!({ "rebuilt": attribute_count, "wiki_rebuilt": wiki_count }),
    ))
}

async fn add_item(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Json(dto): Json<AddItemDto>,
) -> Result<Json<ItemDto>, HttpError> {
    let account = require_account(&actor)?;
    match state
        .services
        .item
        .add_item(&dto, &account)
        .map_err(item_service_err)?
    {
        Some(item) => Ok(Json(ItemDto::from(&item))),
        None => Err(HttpError::Internal),
    }
}

async fn update_item(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(item_id): Path<i64>,
    Json(dto): Json<UpdateItemDto>,
) -> Result<Json<ItemDto>, HttpError> {
    let account = require_account(&actor)?;
    let id = parse_item_id(item_id)?;
    match state
        .services
        .item
        .update_item(&id, &dto, &account)
        .map_err(item_service_err)?
    {
        Some(item) => Ok(Json(ItemDto::from(&item))),
        None => Err(HttpError::NotFound),
    }
}

async fn delete_item(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(item_id): Path<i64>,
) -> Result<StatusCode, HttpError> {
    let account = require_account(&actor)?;
    let id = parse_item_id(item_id)?;
    state
        .services
        .item
        .delete_item(&id, &account)
        .map(|_| StatusCode::OK)
        .map_err(item_service_err)
}

async fn set_attributes(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(item_id): Path<i64>,
    Json(attrs): Json<HashMap<String, Value>>,
) -> Result<StatusCode, HttpError> {
    let account = require_account(&actor)?;
    let id = parse_item_id(item_id)?;
    state
        .services
        .item
        .set_attributes(&id, &attrs, &account)
        .map(|_| StatusCode::OK)
        .map_err(item_service_err)
}

async fn rename_attribute(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(item_id): Path<i64>,
    Json(dto): Json<RenameAttributeDto>,
) -> Result<StatusCode, HttpError> {
    let account = require_account(&actor)?;
    let id = parse_item_id(item_id)?;
    state
        .services
        .item
        .rename_attribute(&id, &dto.old_key, &dto.new_key, &account)
        .map(|_| StatusCode::OK)
        .map_err(item_service_err)
}

async fn delete_attribute(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path((item_id, key)): Path<(i64, String)>,
) -> Result<StatusCode, HttpError> {
    let account = require_account(&actor)?;
    let id = parse_item_id(item_id)?;
    state
        .services
        .item
        .delete_attribute(&id, &key, &account)
        .map(|_| StatusCode::OK)
        .map_err(item_service_err)
}

async fn assign_type(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path((item_id, type_name)): Path<(i64, String)>,
) -> Result<StatusCode, HttpError> {
    let account = require_account(&actor)?;
    let id = parse_item_id(item_id)?;
    state
        .services
        .item
        .assign_type(&type_name, &id, &account)
        .map(|_| StatusCode::OK)
        .map_err(item_service_err)
}

async fn unassign_type(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path((item_id, type_name)): Path<(i64, String)>,
) -> Result<StatusCode, HttpError> {
    let account = require_account(&actor)?;
    let id = parse_item_id(item_id)?;
    state
        .services
        .item
        .unassign_type(&type_name, &id, &account)
        .map(|_| StatusCode::OK)
        .map_err(item_service_err)
}

// ─── Export handlers ──────────────────────────────────────────────────────────

async fn export_pdf(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(item_id): Path<i64>,
) -> Result<Response, HttpError> {
    let account = require_account(&actor)?;
    let id = parse_item_id(item_id)?;
    let item = state
        .services
        .item
        .get_item_by_id(&id, &account)
        .map_err(item_service_err)?
        .ok_or(HttpError::NotFound)?;

    let bytes = generate_pdf(&item).map_err(|e| {
        tracing::error!("PDF export error: {e}");
        HttpError::Internal
    })?;

    let filename = format!("{}.pdf", sanitize_filename(&item.title));
    Ok(download_response(bytes, "application/pdf", &filename))
}

async fn export_docx(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(item_id): Path<i64>,
) -> Result<Response, HttpError> {
    let account = require_account(&actor)?;
    let id = parse_item_id(item_id)?;
    let item = state
        .services
        .item
        .get_item_by_id(&id, &account)
        .map_err(item_service_err)?
        .ok_or(HttpError::NotFound)?;

    let bytes = generate_docx(&item).map_err(|e| {
        tracing::error!("DOCX export error: {e}");
        HttpError::Internal
    })?;

    let filename = format!("{}.docx", sanitize_filename(&item.title));
    Ok(download_response(
        bytes,
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        &filename,
    ))
}

// ─── Export helpers ───────────────────────────────────────────────────────────

fn download_response(bytes: Vec<u8>, content_type: &str, filename: &str) -> Response {
    let disposition = format!("attachment; filename=\"{}\"", filename);
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_DISPOSITION, disposition)
        .body(Body::from(bytes))
        .unwrap()
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim()
        .to_string()
}

/// Strip ZealotScript markup to plain text suitable for PDF/DOCX body.
fn strip_zealotscript(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    let mut chars = content.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            // Skip bold/italic/strikethrough markers
            '*' | '~' => {
                chars.next_if(|&n| n == c);
            }
            '_' => {}
            '\\' => {
                chars.next();
            } // escaped char — skip the backslash
            // Wikilinks [[...]] → just the inner text
            '[' if chars.peek() == Some(&'[') => {
                chars.next(); // consume second [
                let mut inner = String::new();
                loop {
                    match chars.next() {
                        Some(']') if chars.peek() == Some(&']') => {
                            chars.next();
                            break;
                        }
                        Some(ch) => inner.push(ch),
                        None => break,
                    }
                }
                // Strip type prefix "type:Name" → "Name"
                let label = if let Some(pos) = inner.find(':') {
                    &inner[pos + 1..]
                } else {
                    &inner
                };
                out.push_str(label);
            }
            // Regular markdown links [text](url) → text
            '[' => {
                let mut text = String::new();
                loop {
                    match chars.next() {
                        Some(']') => break,
                        Some(ch) => text.push(ch),
                        None => break,
                    }
                }
                // Consume (url) part if present
                if chars.peek() == Some(&'(') {
                    chars.next();
                    loop {
                        match chars.next() {
                            Some(')') | None => break,
                            _ => {}
                        }
                    }
                }
                out.push_str(&text);
            }
            // Heading markers at line start are already stripped by line-level logic
            '#' => {
                out.push(' ');
            }
            '`' => {}
            _ => out.push(c),
        }
    }
    out
}

fn generate_pdf(item: &Item) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use genpdf::{Document, SimplePageDecorator, elements, fonts, style};

    let font_family = fonts::FontFamily {
        regular: fonts::FontData::new(FONT_REGULAR.to_vec(), None)?,
        bold: fonts::FontData::new(FONT_BOLD.to_vec(), None)?,
        italic: fonts::FontData::new(FONT_ITALIC.to_vec(), None)?,
        bold_italic: fonts::FontData::new(FONT_BOLD_ITALIC.to_vec(), None)?,
    };

    let mut doc = Document::new(font_family);
    doc.set_title(&item.title);
    doc.set_minimal_conformance();
    doc.set_line_spacing(1.25);

    let mut decorator = SimplePageDecorator::new();
    decorator.set_margins(10);
    doc.set_page_decorator(decorator);

    // Title
    let title_style = style::Style::new().bold().with_font_size(20);
    doc.push(elements::Paragraph::new(style::StyledString::new(
        item.title.clone(),
        title_style,
    )));
    doc.push(elements::Break::new(0.5));

    // Attributes
    let attrs_json = serde_json::to_value(&item.attributes).unwrap_or(Value::Null);
    if let Value::Object(map) = &attrs_json {
        if !map.is_empty() {
            let label_style = style::Style::new().bold().with_font_size(10);
            let value_style = style::Style::new().with_font_size(10);
            for (k, v) in map {
                let val_str = match v {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                let mut para = elements::Paragraph::new("");
                para.push_styled(format!("{}: ", k), label_style);
                para.push_styled(val_str, value_style);
                doc.push(para);
            }
            doc.push(elements::Break::new(0.5));
        }
    }

    // Content
    let body_style = style::Style::new().with_font_size(11);
    for line in item.content.lines() {
        let trimmed = line.trim_start_matches('#').trim();
        let plain = strip_zealotscript(trimmed);
        if plain.trim().is_empty() {
            doc.push(elements::Break::new(0.3));
        } else {
            doc.push(elements::Paragraph::new(style::StyledString::new(
                plain, body_style,
            )));
        }
    }

    let mut buf = Vec::new();
    doc.render(&mut buf)?;
    Ok(buf)
}

fn generate_docx(item: &Item) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use docx_rs::*;

    let mut doc = Docx::new();

    // Title paragraph
    let title_run = Run::new().add_text(&item.title).bold().size(48); // half-points, so 48 = 24pt
    doc = doc.add_paragraph(Paragraph::new().add_run(title_run));

    // Attributes
    let attrs_json = serde_json::to_value(&item.attributes).unwrap_or(Value::Null);
    if let Value::Object(map) = &attrs_json {
        for (k, v) in map {
            let val_str = match v {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            let key_run = Run::new().add_text(format!("{}: ", k)).bold().size(20);
            let val_run = Run::new().add_text(val_str).size(20);
            doc = doc.add_paragraph(Paragraph::new().add_run(key_run).add_run(val_run));
        }
    }

    // Blank line between attrs and content
    doc = doc.add_paragraph(Paragraph::new());

    // Content
    for line in item.content.lines() {
        let trimmed = line.trim_start_matches('#').trim();
        let plain = strip_zealotscript(trimmed);
        let run = Run::new().add_text(plain).size(22);
        doc = doc.add_paragraph(Paragraph::new().add_run(run));
    }

    let mut buf = Vec::new();
    doc.build().pack(std::io::Cursor::new(&mut buf))?;
    Ok(buf)
}

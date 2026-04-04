use axum::{
    Extension, Json, Router,
    body::Body,
    extract::{Multipart, Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    middleware,
    response::Response,
    routing::{get, patch, post},
};
use sha2::{Digest, Sha256};
use zealot_app::{app::AppState, services::media::MediaServiceError};
use zealot_domain::{
    auth::Actor,
    media::{FileStatDto, MakeFolderDto, RenameMediaDto},
};

use crate::http::{common::HttpError, middleware::auth_middleware};

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        // Specific routes before wildcard routes.
        .route("/mkdir", post(make_folder))
        .route("/rename", patch(rename))
        .route("/*path", get(get_entry).post(upload).delete(delete_entry))
        // Catch root path (no trailing segment).
        .route("/", get(get_root))
        .route_layer(middleware::map_request_with_state(state.clone(), auth_middleware))
        .with_state(state)
}

// ─── Auth helper ──────────────────────────────────────────────────────────────

fn require_account(actor: &Actor) -> Result<zealot_domain::account::Account, HttpError> {
    if !actor.is_authenticated() {
        return Err(HttpError::Unauthorized);
    }
    actor.account.clone().ok_or(HttpError::Unauthorized)
}

fn media_service_err(err: MediaServiceError) -> HttpError {
    match err {
        MediaServiceError::NotFound => HttpError::NotFound,
        MediaServiceError::InvalidPath { reason } => HttpError::UserError { err: reason },
        MediaServiceError::Port(_) => HttpError::Internal,
    }
}

// ─── Response helpers ─────────────────────────────────────────────────────────

#[derive(serde::Serialize)]
struct DirectoryResponse {
    files: Vec<FileStatDto>,
}

fn etag_for_bytes(bytes: &[u8]) -> String {
    let hash = Sha256::digest(bytes);
    hex::encode(hash)
}

fn serve_file(bytes: Vec<u8>, filename: &str, request_headers: &HeaderMap) -> Response {
    let etag = etag_for_bytes(&bytes);

    // If the client already has the current version, return 304.
    if let Some(if_none_match) = request_headers.get(header::IF_NONE_MATCH) {
        if if_none_match.as_bytes() == etag.as_bytes() {
            return Response::builder()
                .status(StatusCode::NOT_MODIFIED)
                .body(Body::empty())
                .unwrap();
        }
    }

    // Guess content type from extension.
    let content_type = mime_guess::from_path(filename)
        .first_raw()
        .unwrap_or("application/octet-stream");

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::ETAG, HeaderValue::from_str(&etag).unwrap_or_else(|_| HeaderValue::from_static("")))
        .body(Body::from(bytes))
        .unwrap()
}

// ─── Handlers ─────────────────────────────────────────────────────────────────

/// GET /media/ — list root directory.
async fn get_root(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    headers: HeaderMap,
) -> Result<Response, HttpError> {
    get_at_path(state, actor, headers, String::new()).await
}

/// GET /media/*path — list directory or download file.
async fn get_entry(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    headers: HeaderMap,
    Path(path): Path<String>,
) -> Result<Response, HttpError> {
    get_at_path(state, actor, headers, path).await
}

async fn get_at_path(
    state: AppState,
    actor: Actor,
    headers: HeaderMap,
    path: String,
) -> Result<Response, HttpError> {
    use zealot_app::services::media::MediaEntry;

    let account = require_account(&actor)?;
    let entry = state
        .services
        .media
        .get(&path, &account)
        .map_err(media_service_err)?;

    match entry {
        MediaEntry::File(file) => {
            let filename = file.stat.name();
            Ok(serve_file(file.bytes, &filename, &headers))
        }
        MediaEntry::Directory(stats) => {
            let files: Vec<FileStatDto> = stats.into_iter().map(FileStatDto::from).collect();
            Ok(axum::response::IntoResponse::into_response(Json(DirectoryResponse { files })))
        }
    }
}

/// POST /media/mkdir — create a folder.
async fn make_folder(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Json(dto): Json<MakeFolderDto>,
) -> Result<StatusCode, HttpError> {
    let account = require_account(&actor)?;
    state
        .services
        .media
        .make_folder(&dto.folder, &account)
        .map(|_| StatusCode::OK)
        .map_err(media_service_err)
}

/// POST /media/*path — upload a file via multipart form.
async fn upload(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(path): Path<String>,
    mut multipart: Multipart,
) -> Result<StatusCode, HttpError> {
    let account = require_account(&actor)?;

    let mut uploaded = false;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| HttpError::UserError { err: e.to_string() })?
    {
        let filename = field
            .file_name()
            .map(String::from)
            .unwrap_or_default();

        if filename.is_empty() {
            continue;
        }

        let bytes = field
            .bytes()
            .await
            .map_err(|e| HttpError::UserError { err: e.to_string() })?;

        state
            .services
            .media
            .upload(&path, &filename, bytes.to_vec(), &account)
            .map_err(media_service_err)?;

        uploaded = true;
    }

    if uploaded {
        Ok(StatusCode::OK)
    } else {
        Err(HttpError::UserError {
            err: String::from("no file field found in multipart body"),
        })
    }
}

/// PATCH /media/rename — rename a file or folder.
async fn rename(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Json(dto): Json<RenameMediaDto>,
) -> Result<StatusCode, HttpError> {
    let account = require_account(&actor)?;
    state
        .services
        .media
        .rename(&dto.old_location, &dto.new_name, &account)
        .map(|_| StatusCode::OK)
        .map_err(media_service_err)
}

/// DELETE /media/*path — delete a file or directory.
async fn delete_entry(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(path): Path<String>,
) -> Result<StatusCode, HttpError> {
    let account = require_account(&actor)?;
    state
        .services
        .media
        .delete(&path, &account)
        .map(|_| StatusCode::OK)
        .map_err(media_service_err)
}


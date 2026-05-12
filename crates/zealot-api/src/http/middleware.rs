use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, Method, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use axum_extra::extract::CookieJar;
use uuid::Uuid;
use zealot_app::app::AppState;
use zealot_domain::auth::{Actor, AuthSource};

pub fn generate_csrf_token() -> String {
    Uuid::new_v4().simple().to_string()
}

pub async fn csrf_middleware(
    State(_state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let is_mutating = !matches!(
        req.method(),
        &Method::GET | &Method::HEAD | &Method::OPTIONS
    );

    if is_mutating {
        let is_session = req
            .extensions()
            .get::<Actor>()
            .map(|a| matches!(a.source, AuthSource::Session))
            .unwrap_or(false);

        if is_session {
            let jar = CookieJar::from_headers(req.headers());
            let cookie_token = jar.get("csfr_").map(|c| c.value().to_string());
            let header_token = req
                .headers()
                .get("x-csrf-token")
                .and_then(|v| v.to_str().ok())
                .map(str::to_string);

            match (header_token, cookie_token) {
                (Some(h), Some(c)) if h == c => {}
                _ => return Err(StatusCode::FORBIDDEN),
            }
        }
    }

    Ok(next.run(req).await)
}

pub async fn auth_middleware<B>(
    State(state): State<AppState>,
    mut req: Request<B>,
) -> Request<B> {
    let headers = req.headers().clone();
    let actor = resolve_actor(&state, headers).await;
    req.extensions_mut().insert(actor);
    req
}

async fn resolve_actor(state: &AppState, headers: HeaderMap) -> Actor {
    // 1. Authorization: Bearer...
    // TODO

    // 2. X-API-Key
    if let Some(api_key) = headers.get("x-api-key") {
        if let Ok(api_key) = api_key.to_str() {
            match state.services.auth.authenticate_api_key(api_key).await {
                Ok(actor) => return actor,
                Err(_) => {
                    // TODO: Handle server errors.
                }
            }
        }
    }

    // 3. Session Management
    let jar = CookieJar::from_headers(&headers);
    if let Some(cookie) = jar.get("session_id") {
        match state.services.auth.authenticate_session(cookie.value()).await {
            Ok(actor) => return actor,
            Err(e) => {
                eprintln!("[AUTH ERROR] session lookup failed: {:?}", e);
            }
        }
    }

    // 4. Anonymous Fallback
    state.services.auth.get_anonymous_actor()
}

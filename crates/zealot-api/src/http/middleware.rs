
use axum::{extract::{Request, State}, http::header, middleware::Next, response::Response};
use axum_extra::extract::CookieJar;
use zealot_app::app::AppState;
use zealot_domain::{account::Account, auth::Actor};


pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Response {
    let actor = resolve_actor(&state, &req).await;
    req.extensions_mut().insert(actor);
    next.run(req).await
}

async fn resolve_actor(state: &AppState, req: &Request) -> Actor {
    // 1. Authorization: Bearer...
    // TODO

    // 2. X-API-Key
    if let Some(api_key) = req.headers().get("x-api-key") {
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
    let jar = CookieJar::from_headers(&req.headers().clone());
    if let Some(cookie) = jar.get("session_id") {
        match state.services.auth.authenticate_session(cookie.value()).await {
            Ok(actor) => return actor,
            Err(_) => {
                // TODO: Distinguish expired/invalid vs internal failure
            }
        }
    }

    // 4. Anonymous Fallback
    state.services.auth.get_anonymous_actor()
}
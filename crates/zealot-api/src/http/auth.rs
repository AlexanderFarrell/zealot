use axum::{Extension, Json, Router, extract::State, middleware, routing::{get, post}};
use zealot_app::{app::AppState, services::auth::AuthError};
use zealot_domain::{account::{AccountDto, LoginBasicDto, RegisterBasicDto}, auth::Actor};

use crate::http::{common::HttpError, middleware::auth_middleware};

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(is_logged_in))
        .route("/is_logged_in", get(is_logged_in))
        .route("/register", post(register_basic))
        .route("/login", post(login_basic))
        .route("/logout", post(logout_basic))
        .route_layer(middleware::map_request_with_state(state.clone(), auth_middleware))
        .with_state(state)
}

async fn is_logged_in(
    Extension(actor): Extension<Actor>,
) -> Result<Json<AccountDto>, HttpError> {
    if actor.is_authenticated() && let Some(account) = actor.account {
        Ok(Json(account.into()))
    } else {
        Err(HttpError::Unauthorized)
    }
}

async fn register_basic(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Json(dto): Json<RegisterBasicDto>
) -> Result<Json<AccountDto>, HttpError> {
    if actor.is_authenticated() {
        return Err(HttpError::UserError { err: String::from("Already logged in") });
    }

    match state.services.auth.register_account(&dto).await {
        Ok(account) => Ok(Json(account.into())),
        Err(error) => match error {
            AuthError::RegisterError { err } => Err(HttpError::UserError { err }),
            AuthError::ServerError => Err(HttpError::Internal),
            AuthError::LoginError { .. } => Err(HttpError::Internal),
        }
    }
}

async fn login_basic(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Json(dto): Json<LoginBasicDto>
) -> Result<Json<AccountDto>, HttpError> {
    if actor.is_authenticated() {
        return Err(HttpError::UserError { err: String::from("Already logged in") });
    }

    match state.services.auth.login_account(&dto).await {
        Ok(account) => Ok(Json(account.into())),
        Err(error) => match error {
            AuthError::LoginError { err } => Err(HttpError::UserError { err }),
            AuthError::ServerError => Err(HttpError::Internal),
            AuthError::RegisterError { .. } => Err(HttpError::Internal),
        },
    }
}

async fn logout_basic(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>
) -> Result<(), HttpError> {
    if !actor.is_authenticated() {
        return Err(HttpError::UserError { err: String::from("Not logged in") });
    }

    match state.services.auth.logout_account(&actor).await {
        Ok(_) => Ok(()),
        Err(_) => Err(HttpError::Internal),
    }
}

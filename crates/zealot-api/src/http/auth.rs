use axum::{Json, Router, extract::State, middleware, routing::{get, post}};
use zealot_app::{app::AppState, repos::account, services::{auth::AuthError, common::ServiceError}};
use zealot_domain::{account::{AccountDto, LoginBasicDto, RegisterBasicDto}, auth::Actor};

use crate::http::common::HttpError;

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(is_logged_in))
        .route("/is_logged_in", get(is_logged_in))
        .route("/register", post(register_basic))
        .route("/login", post(login_basic))
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
        .with_state(state)
}

async fn is_logged_in(
    State(state): State<AppState>,
    actor: Actor,
) -> Result<Json<AccountDto>, HttpError> {
    if actor.is_authenticated() && let Some(account) = actor.account {
        Ok(Json(account))
    } else {
        Err(HttpError::Unauthorized)
    }
}

async fn register_basic(
    State(state): State<AppState>,
    actor: Actor,
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
            _ => Err(HttpError::Internal) // TODO: I don't like this, handle another way
        }
    }
}

async fn login_basic(
    State(state): State<AppState>,
    actor: Actor,
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
            _ => Err(HttpError::Internal) // TODO: I don't like this, handle another way.
        },
    }
}
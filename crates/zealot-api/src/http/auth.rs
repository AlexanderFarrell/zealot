use axum::{
    Extension, Json, Router,
    extract::State,
    middleware,
    routing::{get, post},
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use time::Duration;
use zealot_app::{app::AppState, services::auth::AuthError};
use zealot_domain::{
    account::{
        AccountDto, CreateApiKeyResponseDto, CreateApiKeyWithCredentialsDto, LoginBasicDto,
        RegisterBasicDto,
    },
    auth::Actor,
};

use crate::http::{
    common::HttpError,
    middleware::{auth_middleware, generate_csrf_token},
};

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(is_logged_in))
        .route("/is_logged_in", get(is_logged_in))
        .route("/register", post(register_basic))
        .route("/login", post(login_basic))
        .route("/logout", post(logout_basic))
        .route_layer(middleware::map_request_with_state(
            state.clone(),
            auth_middleware,
        ))
        .route("/api_key", post(create_api_key_with_credentials))
        .with_state(state)
}

async fn is_logged_in(
    Extension(actor): Extension<Actor>,
    jar: CookieJar,
) -> Result<(CookieJar, Json<AccountDto>), HttpError> {
    if actor.is_authenticated()
        && let Some(account) = actor.account
    {
        let csrf_cookie = Cookie::build(("csfr_", generate_csrf_token()))
            .http_only(false)
            .same_site(SameSite::Lax)
            .path("/")
            .max_age(Duration::days(30))
            .build();
        Ok((jar.add(csrf_cookie), Json(account.into())))
    } else {
        Err(HttpError::Unauthorized)
    }
}

async fn register_basic(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    jar: CookieJar,
    Json(dto): Json<RegisterBasicDto>,
) -> Result<(CookieJar, Json<AccountDto>), HttpError> {
    if actor.is_authenticated() {
        return Err(HttpError::UserError {
            err: String::from("Already logged in"),
        });
    }

    match state.services.auth.register_account(&dto).await {
        Ok((account, raw_token)) => {
            tracing::info!(account_id = ?account.account_id, username = %dto.username, "account registered");
            let cookie = Cookie::build(("session_id", raw_token))
                .http_only(true)
                .same_site(SameSite::Lax)
                .path("/")
                .max_age(Duration::days(30))
                .build();
            let csrf_cookie = Cookie::build(("csfr_", generate_csrf_token()))
                .http_only(false)
                .same_site(SameSite::Lax)
                .path("/")
                .max_age(Duration::days(30))
                .build();
            Ok((jar.add(cookie).add(csrf_cookie), Json(account.into())))
        }
        Err(error) => match error {
            AuthError::RegisterError { err } => Err(HttpError::UserError { err }),
            AuthError::ServerError => {
                tracing::error!(username = %dto.username, "server error during register");
                Err(HttpError::Internal)
            }
            AuthError::LoginError { .. } => {
                tracing::error!(username = %dto.username, "unexpected login error during register");
                Err(HttpError::Internal)
            }
        },
    }
}

async fn login_basic(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    jar: CookieJar,
    Json(dto): Json<LoginBasicDto>,
) -> Result<(CookieJar, Json<AccountDto>), HttpError> {
    if actor.is_authenticated() {
        return Err(HttpError::UserError {
            err: String::from("Already logged in"),
        });
    }

    match state.services.auth.login_account(&dto).await {
        Ok((account, raw_token)) => {
            tracing::info!(account_id = ?account.account_id, username = %dto.username, "account logged in");
            let cookie = Cookie::build(("session_id", raw_token))
                .http_only(true)
                .same_site(SameSite::Lax)
                .path("/")
                .max_age(Duration::days(30))
                .build();
            let csrf_cookie = Cookie::build(("csfr_", generate_csrf_token()))
                .http_only(false)
                .same_site(SameSite::Lax)
                .path("/")
                .max_age(Duration::days(30))
                .build();
            Ok((jar.add(cookie).add(csrf_cookie), Json(account.into())))
        }
        Err(error) => match error {
            AuthError::LoginError { err } => Err(HttpError::UserError { err }),
            AuthError::ServerError => {
                tracing::error!(username = %dto.username, "server error during login");
                Err(HttpError::Internal)
            }
            AuthError::RegisterError { .. } => {
                tracing::error!(username = %dto.username, "unexpected register error during login");
                Err(HttpError::Internal)
            }
        },
    }
}

async fn create_api_key_with_credentials(
    State(state): State<AppState>,
    Json(dto): Json<CreateApiKeyWithCredentialsDto>,
) -> Result<Json<CreateApiKeyResponseDto>, HttpError> {
    let login_dto = LoginBasicDto {
        username: dto.username,
        password: dto.password,
    };
    match state.services.auth.login_account(&login_dto).await {
        Ok((account, _token)) => {
            let label = dto.label.unwrap_or_else(|| "Mobile".to_string());
            let (record, raw_key) = state
                .services
                .account
                .generate_api_key(&account.account_id, &label)
                .map_err(|e| {
                    tracing::error!(account_id = ?account.account_id, %e, "failed to generate api key via credentials");
                    HttpError::Internal
                })?;
            tracing::info!(account_id = ?account.account_id, "api key created via credentials");
            Ok(Json(CreateApiKeyResponseDto {
                key: raw_key,
                api_key_id: record.api_key_id.into(),
                label: record.label,
                created_at: record.created_at,
            }))
        }
        Err(error) => match error {
            AuthError::LoginError { err } => Err(HttpError::UserError { err }),
            AuthError::ServerError => {
                tracing::error!(username = %login_dto.username, "server error during api key credential login");
                Err(HttpError::Internal)
            }
            AuthError::RegisterError { .. } => {
                tracing::error!(username = %login_dto.username, "unexpected register error during api key credential login");
                Err(HttpError::Internal)
            }
        },
    }
}

async fn logout_basic(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    jar: CookieJar,
) -> Result<CookieJar, HttpError> {
    if !actor.is_authenticated() {
        return Err(HttpError::UserError {
            err: String::from("Not logged in"),
        });
    }

    let session_token = jar.get("session_id").map(|c| c.value().to_string());

    match state
        .services
        .auth
        .logout_account(&actor, session_token.as_deref())
        .await
    {
        Ok(_) => {
            let cleared = Cookie::build(("session_id", ""))
                .path("/")
                .max_age(Duration::ZERO)
                .build();
            let csrf_cleared = Cookie::build(("csfr_", ""))
                .path("/")
                .max_age(Duration::ZERO)
                .build();
            Ok(jar.remove(cleared).remove(csrf_cleared))
        }
        Err(e) => {
            tracing::error!(%e, "failed to logout account");
            Err(HttpError::Internal)
        }
    }
}

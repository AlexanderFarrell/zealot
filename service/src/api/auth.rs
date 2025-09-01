//! OAuth2 authentication for login, logout, and renewal of tokens

use actix_web::{get, web, Scope, HttpResponse};

#[get("")]
async fn index() -> HttpResponse {
    HttpResponse::Ok().body("auth")
}

pub fn auth_scope() -> Scope {
    web::scope("/auth")
        .service(index)
}
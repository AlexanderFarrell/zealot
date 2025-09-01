//! Change zealot settings

use actix_web::{get, web, Scope, HttpResponse};

#[get("")]
async fn index() -> HttpResponse {
    HttpResponse::Ok().body("settings")
}

pub fn settings_scope() -> Scope {
    web::scope("/settings")
        .service(index)
}
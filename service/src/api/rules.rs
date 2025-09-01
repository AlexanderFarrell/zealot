//! CRUD for rules engine

use actix_web::{get, web, Scope, HttpResponse};

#[get("")]
async fn index() -> HttpResponse {
    HttpResponse::Ok().body("rules")
}

pub fn rules_scope() -> Scope {
    web::scope("/rules")
        .service(index)
}
//! Reports on issues, such as items not adhering to schema or orphan items

use actix_web::{get, web, Scope, HttpResponse};

#[get("")]
async fn index() -> HttpResponse {
    HttpResponse::Ok().body("audit")
}

pub fn audit_scope() -> Scope {
    web::scope("/audit")
        .service(index)
}
//! Reports on issues, such as items not adhering to schema or orphan items

use actix_web::{get, web, Scope, HttpResponse};

/// TODO
#[get("")]
async fn index() -> HttpResponse {
    HttpResponse::Ok().body("audit")
}

/// TODO
pub fn audit_scope() -> Scope {
    web::scope("/audit")
        .service(index)
}
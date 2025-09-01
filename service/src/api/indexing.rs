//! Gets indexed items, like links, headings, etc.

use actix_web::{get, web, Scope, HttpResponse};

/// TODO
#[get("")]
async fn index() -> HttpResponse {
    HttpResponse::Ok().body("indexing")
}

/// TODO
pub fn indexing_scope() -> Scope {
    web::scope("/indexing")
        .service(index)
}
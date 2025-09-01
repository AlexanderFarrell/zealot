//! Gets indexed items, like links, headings, etc.

use actix_web::{get, web, Scope, HttpResponse};

#[get("")]
async fn index() -> HttpResponse {
    HttpResponse::Ok().body("indexing")
}

pub fn indexing_scope() -> Scope {
    web::scope("/indexing")
        .service(index)
}
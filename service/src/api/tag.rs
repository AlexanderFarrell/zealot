//! CRUD for tags (might merge with item)

use actix_web::{get, web, Scope, HttpResponse};

#[get("")]
async fn index() -> HttpResponse {
    HttpResponse::Ok().body("tags")
}

pub fn tag_scope() -> Scope {
    web::scope("/tag")
        .service(index)
}
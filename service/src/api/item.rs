//! CRUD operations on items. Search

use actix_web::{get, web, Scope, HttpResponse};

#[get("")]
async fn index() -> HttpResponse {
    HttpResponse::Ok().body("list of items")
}

pub fn item_scope() -> Scope {
    web::scope("/item")
        .service(index)
}
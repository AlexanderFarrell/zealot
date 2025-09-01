//! Upload and download of media, including pictures, audio, epub, JSON, etc.

use actix_web::{get, web, Scope, HttpResponse};

#[get("")]
async fn index() -> HttpResponse {
    HttpResponse::Ok().body("media")
}

pub fn media_scope() -> Scope {
    web::scope("/media")
        .service(index)
}
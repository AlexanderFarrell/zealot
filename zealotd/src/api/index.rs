use actix_web::{Scope, web, HttpResponse, get};

#[get("")]
pub async fn index_endpoint() -> Result<HttpResponse, actix_web::Error> {
    return Ok(HttpResponse::Ok().body("Hello!"));
}

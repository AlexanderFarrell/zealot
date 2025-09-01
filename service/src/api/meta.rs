use actix_web::{get, web, Scope, HttpResponse};

/// TODO
#[get("")]
async fn index() -> HttpResponse {
    HttpResponse::Ok().body("meta")
}

/// TODO
pub fn meta_scope() -> Scope {
    web::scope("/meta")
        .service(index)
}
use actix_web::{get, web, Scope, HttpResponse};

#[get("")]
async fn index() -> HttpResponse {
    HttpResponse::Ok().body("meta")
}

pub fn meta_scope() -> Scope {
    web::scope("/meta")
        .service(index)
}
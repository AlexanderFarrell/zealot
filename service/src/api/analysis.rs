//! Scorecard and other analytics

use actix_web::{get, web, Scope, HttpResponse};

#[get("")]
async fn index() -> HttpResponse {
    HttpResponse::Ok().body("analysis")
}

pub fn analysis_scope() -> Scope {
    web::scope("/analysis")
        .service(index)
}
//! Scorecard and other analytics

use actix_web::{get, web, Scope, HttpResponse};

/// TODO
#[get("")]
async fn index() -> HttpResponse {
    HttpResponse::Ok().body("analysis")
}

/// TODO
pub fn analysis_scope() -> Scope {
    web::scope("/analysis")
        .service(index)
}
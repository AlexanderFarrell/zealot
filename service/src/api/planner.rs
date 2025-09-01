//! Daily, weekly, monthly, quarterly and annual planners.

use actix_web::{get, web, Scope, HttpResponse};

/// TODO
#[get("")]
async fn index() -> HttpResponse {
    HttpResponse::Ok().body("planner")
}

/// TODO
pub fn planner_scope() -> Scope {
    web::scope("/planner")
        .service(index)
}
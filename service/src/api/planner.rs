//! Daily, weekly, monthly, quarterly and annual planners.

use actix_web::{get, web, Scope, HttpResponse};

#[get("")]
async fn index() -> HttpResponse {
    HttpResponse::Ok().body("planner")
}

pub fn planner_scope() -> Scope {
    web::scope("/planner")
        .service(index)
}
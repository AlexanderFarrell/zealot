//! Main entry point for the zealotd service. 

use actix_web::{get, App, HttpResponse, HttpServer, Responder};

pub mod api;
pub mod core;

/// TODO
#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello World")
}

/// TODO
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let sql = String::from(include_str!("../../database/item.sql"));
    if let Err(err) = core::database::database_seed(&sql) {
        panic!("{}", err);
    };

    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(api::analysis::analysis_scope())
            .service(api::audit::audit_scope())
            .service(api::auth::auth_scope())
            .service(api::indexing::indexing_scope())
            .service(api::item::item_scope())
            .service(api::media::media_scope())
            .service(api::meta::meta_scope())
            .service(api::planner::planner_scope())
            .service(api::rules::rules_scope())
            .service(api::settings::settings_scope())
            .service(api::tag::tag_scope())
    })
    .bind(("0.0.0.0", 8082))?
    .workers(1) // When running as a background process, we only care about this.
    .run()
    .await
}
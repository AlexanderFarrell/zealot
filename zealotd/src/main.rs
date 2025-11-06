use actix_web::{get, App, HttpResponse, HttpServer, Responder};

pub mod api;
pub mod core;
pub mod data;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let hostname = ("0.0.0.0", 8082);

    let sql = String::from(include_str!("../../database/init.sql"));
    if let Err(err) = core::database::database_seed(&sql) {
        panic!("{}", err);
    }

    let server = HttpServer::new(|| {
        App::new()
            .service(api::index::index_endpoint)
            .service(api::item::item_scope())
    })
    .bind(hostname)?
    .workers(1); // When running as a desktop daemon, we spawn one worker. When running as a web server, we spawn many.

    println!("Server is running at http://{}:{}", hostname.0, hostname.1);

    server.run().await
}
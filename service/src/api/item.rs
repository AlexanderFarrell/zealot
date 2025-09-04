//! CRUD operations on items. Search

use actix_web::{get, post, web, Scope, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct AddInfo {
    title: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Item {
    id: u32,
    title: String,
}

/// TODO
#[get("")]
async fn index() -> Result<HttpResponse, actix_web::Error> {
    // Change this later, we don't want to get all items.
    let db = crate::core::database::DB.lock().unwrap();
    let mut stmt = db
        .prepare("SELECT item_id, title FROM item;")
        .map_err(|e| {
            eprintln!("DB prepare error: {}", e);
            actix_web::error::ErrorInternalServerError("DB Error")
        })?;
    let items = stmt
        .query_map([], |row| {
            Ok(Item {
                id: row.get(0)?,
                title: row.get(1)?,
            })
        })
        .map_err(|e| {
            eprintln!("DB Query Error?: {}", e);
            actix_web::error::ErrorInternalServerError("DB Error")
        })?
        .filter_map(|res| res.ok())
        .collect::<Vec<Item>>();

    Ok(HttpResponse::Ok().json(items))
}

#[post("")]
async fn add(add_info: web::Json<AddInfo>) -> HttpResponse {
    let db = crate::core::database::DB.lock().unwrap();
    let e = db.execute(
        "INSERT INTO item (title) values (?1);", 
        (&add_info.title,));

    match e {
        Err(ee) => {
            eprintln!("Error adding item: {}", ee);
            HttpResponse::InternalServerError().body("Error adding item")
        },
        Ok(_) => {
            HttpResponse::Ok().body("Success")
        }
    }
}

/// TODO
pub fn item_scope() -> Scope {
    web::scope("/item")
        .service(index)
        .service(add)
}
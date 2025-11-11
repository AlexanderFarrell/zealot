//! CRUD operations on items. Search

use actix_web::{get, post, delete, web::{self, get}, HttpResponse, Scope};
use crate::data::item::{ItemDBO, AddInfo};

/// TODO
#[get("")]
async fn index() -> Result<HttpResponse, actix_web::Error> {
    match ItemDBO::get_recent(1, 20).await {
        Ok(items) => {
            Ok(HttpResponse::Ok().json(items))
        },
        Err(_) => {
            Ok(HttpResponse::InternalServerError().body("Error getting recent"))
        }
    }
}

#[get("/id/{item_id}")]
async fn get_by_id(path: web::Path<u32>) -> Result<HttpResponse, actix_web::Error> {
    let item_id = path.into_inner();
    match ItemDBO::get_by_id(item_id).await {
        Ok(Some(item)) => {
            Ok(HttpResponse::Ok().json(item))
        },
        Ok(None) => {
            Ok(HttpResponse::NotFound().body(""))
        },
        Err(err) => {
            eprintln!("Error getting item: {err}");
            Ok(HttpResponse::InternalServerError().body(""))
        }
    }
}

#[get("/title/{title}")]
async fn get_by_title(path: web::Path<String>) -> Result<HttpResponse, actix_web::Error> {
    let title = path.into_inner();
    match ItemDBO::get_by_title(title.as_str()).await {
        Ok(Some(item)) => {
            Ok(HttpResponse::Ok().json(item))
        },
        Ok(None) => {
            Ok(HttpResponse::NotFound().body(""))
        },
        Err(err) => {
            eprintln!("Error getting item: {err}");
            Ok(HttpResponse::InternalServerError().body(""))
        }
    }
}

#[get("/search/{term}")]
async fn search_by_title(path: web::Path<String>) -> Result<HttpResponse, actix_web::Error> {
    let term = path.into_inner();
    match ItemDBO::search_by_title(&term.as_str(), 20).await {
        Ok(items) => {
            Ok(HttpResponse::Ok().json(items))
        },
        Err(_) => {
            Ok(HttpResponse::InternalServerError().body("Error searching"))
        }
    }
}

#[post("")]
async fn add(add_info: web::Json<AddInfo>) -> Result<HttpResponse, actix_web::Error> {
    match ItemDBO::add(&add_info).await {
        Ok(_) => Ok(HttpResponse::Ok().body("")),
        Err(_) => Ok(HttpResponse::InternalServerError().body("Error adding item")),
    }
}

#[delete("")]
async fn remove(path: web::Path<u32>) -> Result<HttpResponse, actix_web::Error> {
    let item_id = path.into_inner();
    match ItemDBO::delete(&item_id).await {
        Ok(_) => Ok(HttpResponse::Ok().body("")),
        Err(_) => Ok(HttpResponse::InternalServerError().body("Error removing item")),
    }
}

/// TODO
pub fn item_scope() -> Scope {
    web::scope("/item")
        .service(index)
        .service(get_by_id)
        .service(get_by_title)
        .service(search_by_title)
        .service(add)
        .service(remove)
}
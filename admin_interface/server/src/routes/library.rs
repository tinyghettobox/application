use actix_web::{delete, get, post, put, web, Responder};
use tracing::{error, info};
use database::{DatabaseConnection, DbErr, LibraryEntryRepository};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct GetParams {
    pub id: Option<i32>,
}

#[get("/api/library/{id}")]
pub async fn get(conn: web::Data<DatabaseConnection>, params: web::Path<GetParams>) -> impl Responder {
    let id = params.id;
    info!("Getting library entry: {:?}", id);
    match LibraryEntryRepository::get(&conn, id.unwrap_or(0)).await {
        Ok(model) => match model {
            Some(model) => {
                actix_web::HttpResponse::Ok().json(model)
            },
            None => actix_web::HttpResponse::NotFound().finish(),
        },
        Err(error) => match error {
            DbErr::Json(msg) => actix_web::HttpResponse::BadRequest().body(msg),
            DbErr::RecordNotFound(_) => actix_web::HttpResponse::NotFound().finish(),
            _ => {
                error!("Failed to get library entry: {:?}", error);
                actix_web::HttpResponse::InternalServerError().finish()
            }
        },
    }
}

#[put("/api/library/{id}")]
pub async fn update(
    conn: web::Data<DatabaseConnection>,
    id: web::Path<i32>,
    entry: web::Json<serde_json::Value>,
) -> impl Responder {
    let id = id.into_inner();
    match LibraryEntryRepository::update(&conn, id, entry.into_inner()).await {
        Ok(model) => actix_web::HttpResponse::Ok().json(model),
        Err(error) => match error {
            DbErr::Json(msg) => actix_web::HttpResponse::BadRequest().body(msg),
            DbErr::RecordNotFound(_) => actix_web::HttpResponse::NotFound().finish(),
            _ => {
                error!("Failed to update library entry: {:?}", error);
                actix_web::HttpResponse::InternalServerError().finish()
            }
        },
    }
}

#[derive(Deserialize)]
pub struct PostQuery {
    parent_id: Option<i32>,
}

#[post("/api/library")]
pub async fn create(
    conn: web::Data<DatabaseConnection>,
    query: web::Query<PostQuery>,
    entry: web::Json<serde_json::Value>,
) -> impl Responder {
    match LibraryEntryRepository::create(&conn, query.parent_id, entry.into_inner()).await {
        Ok(model) => actix_web::HttpResponse::Ok().json(model),
        Err(error) => match error {
            DbErr::Json(msg) => actix_web::HttpResponse::BadRequest().body(msg),
            DbErr::RecordNotFound(_) => actix_web::HttpResponse::NotFound().finish(),
            _ => {
                error!("Failed to create library entry: {:?}", error);
                actix_web::HttpResponse::InternalServerError().finish()
            }
        },
    }
}

#[delete("/api/library/{id}")]
pub async fn delete(conn: web::Data<DatabaseConnection>, id: web::Path<i32>) -> impl Responder {
    match LibraryEntryRepository::delete(&conn, id.into_inner()).await {
        Ok(deleted) => {
            if deleted {
                actix_web::HttpResponse::Ok().finish()
            } else {
                actix_web::HttpResponse::NotFound().finish()
            }
        }
        Err(error) => match error {
            DbErr::Json(msg) => actix_web::HttpResponse::BadRequest().body(msg),
            DbErr::RecordNotFound(_) => actix_web::HttpResponse::NotFound().finish(),
            _ => {
                error!("Failed to delete library entry: {:?}", error);
                actix_web::HttpResponse::InternalServerError().finish()
            }
        },
    }
}

use crate::file_cache::FileCache;
use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::text::Text;
use actix_multipart::form::MultipartForm;
use actix_web::{delete, get, post, put, web, Responder};
use database::{
    model::library_entry::CreateModel as LibraryEntryCreateModel, model::library_entry::Model as LibraryEntry,
    DatabaseConnection, DbErr, LibraryEntryRepository,
};
use serde::Deserialize;
use tracing::{error, info};

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
            Some(model) => actix_web::HttpResponse::Ok().json(model),
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
    entry: web::Json<LibraryEntry>,
) -> impl Responder {
    let id = id.into_inner();
    let entry = entry.into_inner();

    match LibraryEntryRepository::update(&conn, id, entry).await {
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

#[derive(MultipartForm)]
pub struct UploadForm {
    name: Text<String>,
    #[multipart(limit = "300MB")]
    track: TempFile,
}

#[post("/api/library/upload")]
pub async fn upload(
    file_cache: web::Data<FileCache>,
    MultipartForm(form): MultipartForm<UploadForm>,
) -> impl Responder {
    std::fs::read(form.track.file.path())
        .map_err(|error| format!("could not read temp file: {}", error))
        .and_then(|binary| file_cache.add(form.name.clone(), binary.clone()))
        .map(|_| actix_web::HttpResponse::Ok().finish())
        .unwrap_or_else(|error| actix_web::HttpResponse::BadRequest().body(error))
}

#[derive(Deserialize)]
pub struct PostQuery {
    parent_id: Option<i32>,
}

#[post("/api/library")]
pub async fn create(
    conn: web::Data<DatabaseConnection>,
    file_cache: web::Data<FileCache>,
    query: web::Query<PostQuery>,
    entries: web::Json<Vec<LibraryEntryCreateModel>>,
) -> impl Responder {
    let entries = match enrich_entries_with_cached_files(&file_cache, entries.into_inner()) {
        Ok(entries) => entries,
        Err(error) => {
            return actix_web::HttpResponse::BadRequest().body(error);
        }
    };

    match LibraryEntryRepository::create(&conn, query.parent_id, entries).await {
        Ok(models) => actix_web::HttpResponse::Ok().json(models),
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

fn enrich_entries_with_cached_files(
    file_cache: &web::Data<FileCache>,
    entries: Vec<LibraryEntryCreateModel>,
) -> Result<Vec<LibraryEntryCreateModel>, String> {
    let mut new_entries = vec![];
    for mut entry in entries.clone() {
        if let Some(children) = entry.children.as_mut() {
            entry.children = Some(enrich_entries_with_cached_files(file_cache, children.clone())?);
        }
        if let Some(track_source) = entry.track_source.as_mut() {
            if track_source.file.is_some() {
                continue;
            }

            let uploaded_file = file_cache.get(track_source.title.clone())?;
            track_source.file = Some(uploaded_file);
        }
        new_entries.push(entry);
    }
    Ok(new_entries)
}

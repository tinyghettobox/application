use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::text::Text;
use actix_multipart::form::MultipartForm;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use database::model::track_source::CreateModel;
use database::{
    model::library_entry::CreateModel as LibraryEntryCreateModel,
    model::library_entry::Model as LibraryEntry, DatabaseConnection, DbErr, LibraryEntryRepository,
    TrackSourceRepository,
};
use serde::Deserialize;
use tracing::{error, info};

#[derive(Deserialize)]
pub struct GetParams {
    pub id: Option<i32>,
}

#[get("/api/library/{id}")]
pub async fn get(
    conn: web::Data<DatabaseConnection>,
    params: web::Path<GetParams>,
) -> impl Responder {
    let id = params.id;
    info!("Getting library entry: {:?}", id);
    match LibraryEntryRepository::get(&conn, id.unwrap_or(0)).await {
        Ok(model) => match model {
            Some(model) => HttpResponse::Ok().json(model),
            None => HttpResponse::NotFound().finish(),
        },
        Err(error) => match error {
            DbErr::Json(msg) => HttpResponse::BadRequest().body(msg),
            DbErr::RecordNotFound(_) => HttpResponse::NotFound().finish(),
            _ => {
                error!("Failed to get library entry: {:?}", error);
                HttpResponse::InternalServerError().finish()
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
        Ok(model) => HttpResponse::Ok().json(model),
        Err(error) => match error {
            DbErr::Json(msg) => HttpResponse::BadRequest().body(msg),
            DbErr::RecordNotFound(_) => HttpResponse::NotFound().finish(),
            _ => {
                error!("Failed to update library entry: {:?}", error);
                HttpResponse::InternalServerError().finish()
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
    conn: web::Data<DatabaseConnection>,
    MultipartForm(form): MultipartForm<UploadForm>,
) -> impl Responder {
    let binary = match std::fs::read(form.track.file.path()) {
        Ok(binary) => binary,
        Err(error) => {
            return HttpResponse::BadRequest().body(format!("could not read temp file: {}", error));
        }
    };

    let track_source = TrackSourceRepository::create(
        conn.as_ref(),
        None,
        CreateModel::new_file(form.name.clone(), binary),
    )
    .await;

    match track_source {
        Ok(source) => HttpResponse::Ok().json(source),
        Err(error) => {
            HttpResponse::BadRequest().body(format!("could not create track source: {}", error))
        }
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
    entries: web::Json<Vec<LibraryEntryCreateModel>>,
) -> impl Responder {
    match LibraryEntryRepository::create(&conn, query.parent_id, entries.into_inner()).await {
        Ok(models) => HttpResponse::Ok().json(models),
        Err(error) => match error {
            DbErr::Json(msg) => HttpResponse::BadRequest().body(msg),
            DbErr::RecordNotFound(_) => HttpResponse::NotFound().finish(),
            _ => {
                error!("Failed to create library entry: {:?}", error);
                HttpResponse::InternalServerError().finish()
            }
        },
    }
}

#[delete("/api/library/{id}")]
pub async fn delete(conn: web::Data<DatabaseConnection>, id: web::Path<i32>) -> impl Responder {
    match LibraryEntryRepository::delete(&conn, id.into_inner()).await {
        Ok(deleted) => {
            if deleted {
                HttpResponse::Ok().finish()
            } else {
                HttpResponse::NotFound().finish()
            }
        }
        Err(error) => match error {
            DbErr::Json(msg) => HttpResponse::BadRequest().body(msg),
            DbErr::RecordNotFound(_) => HttpResponse::NotFound().finish(),
            _ => {
                error!("Failed to delete library entry: {:?}", error);
                HttpResponse::InternalServerError().finish()
            }
        },
    }
}

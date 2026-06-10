use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::text::Text;
use actix_multipart::form::MultipartForm;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use std::sync::Arc;
use tokio::sync::Notify;
use chrono::{DateTime, Utc};
use database::model::library_entry::BulkUpdateModel;
use database::model::sync_config::CreateSyncConfig;
use database::model::track_source::CreateModel;
use database::{
    model::library_entry::CreateModel as LibraryEntryCreateModel,
    model::library_entry::Model as LibraryEntry, DatabaseConnection, DbErr, LibraryEntryRepository,
    SyncConfigRepository, SyncStatusRepository, TrackSourceRepository,
};
use serde::{Deserialize, Serialize};
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

#[post("/api/library/bulk-update")]
pub async fn bulk_update(
    conn: web::Data<DatabaseConnection>,
    updates: web::Json<Vec<BulkUpdateModel>>,
) -> impl Responder {
    match LibraryEntryRepository::bulk_patch(&conn, updates.into_inner()).await {
        Ok(results) => HttpResponse::Ok().json(results),
        Err(error) => {
            error!("Failed to apply bulk updates: {:?}", error);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(Deserialize, Serialize)]
struct MarkPlayedPayload {
    library_entry_ids: Vec<i32>,
    played_at: Option<DateTime<Utc>>,
}

#[post("/api/library/mark-played")]
pub async fn mark_played(
    conn: web::Data<DatabaseConnection>,
    payload: web::Json<MarkPlayedPayload>,
) -> impl Responder {
    match LibraryEntryRepository::mark_played(
        &conn,
        payload.library_entry_ids.clone(),
        payload.played_at,
    )
    .await
    {
        Ok(results) => HttpResponse::Ok().json(results),
        Err(error) => {
            error!("Failed to mark as played: {:?}", error);
            HttpResponse::InternalServerError().finish()
        }
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
    notify: web::Data<Arc<Notify>>,
    query: web::Query<PostQuery>,
    entries: web::Json<Vec<LibraryEntryCreateModel>>,
) -> impl Responder {
    match LibraryEntryRepository::create(&conn, query.parent_id, entries.into_inner()).await {
        Ok(models) => {
            notify.notify_one();
            HttpResponse::Ok().json(models)
        }
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
    let id = id.into_inner();
    match LibraryEntryRepository::delete(&conn, id).await {
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

// ---------------------------------------------------------------------------
// Sync config endpoints
// ---------------------------------------------------------------------------

#[get("/api/library/{id}/sync-config")]
pub async fn get_sync_config(
    conn: web::Data<DatabaseConnection>,
    id: web::Path<i32>,
) -> impl Responder {
    match SyncConfigRepository::get(&conn, id.into_inner()).await {
        Ok(Some(config)) => HttpResponse::Ok().json(config),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(e) => {
            error!("Failed to get sync config: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[put("/api/library/{id}/sync-config")]
pub async fn upsert_sync_config(
    conn: web::Data<DatabaseConnection>,
    notify: web::Data<Arc<Notify>>,
    id: web::Path<i32>,
    body: web::Json<CreateSyncConfig>,
) -> impl Responder {
    let library_entry_id = id.into_inner();
    match SyncConfigRepository::upsert(&conn, library_entry_id, body.into_inner()).await {
        Ok(config) => {
            // Mark as pending so it gets picked up by the sync job.
            let _ = SyncStatusRepository::set_pending(&conn, library_entry_id).await;
            notify.notify_one();
            HttpResponse::Ok().json(config)
        }
        Err(e) => {
            error!("Failed to upsert sync config: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/api/library/{id}/children/sync-statuses")]
pub async fn get_children_sync_statuses(
    conn: web::Data<DatabaseConnection>,
    id: web::Path<i32>,
) -> impl Responder {
    match SyncStatusRepository::get_for_parent(&conn, id.into_inner()).await {
        Ok(statuses) => HttpResponse::Ok().json(statuses),
        Err(e) => {
            error!("Failed to get children sync statuses: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/api/library/{id}/sync-status")]
pub async fn get_sync_status(
    conn: web::Data<DatabaseConnection>,
    id: web::Path<i32>,
) -> impl Responder {
    match SyncStatusRepository::get(&conn, id.into_inner()).await {
        Ok(Some(status)) => HttpResponse::Ok().json(status),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(e) => {
            error!("Failed to get sync status: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

/// Trigger an immediate sync for the given library entry.
/// Sets status = pending and wakes the sync job so it starts within seconds.
#[post("/api/library/{id}/sync")]
pub async fn trigger_sync(
    conn: web::Data<DatabaseConnection>,
    notify: web::Data<Arc<Notify>>,
    id: web::Path<i32>,
) -> impl Responder {
    let library_entry_id = id.into_inner();
    // Verify a sync config exists first.
    match SyncConfigRepository::get(&conn, library_entry_id).await {
        Ok(None) => return HttpResponse::BadRequest().body("No sync config for this entry"),
        Err(e) => {
            error!("Failed to check sync config: {:?}", e);
            return HttpResponse::InternalServerError().finish();
        }
        _ => {}
    }
    match SyncStatusRepository::set_pending(&conn, library_entry_id).await {
        Ok(_) => {
            notify.notify_one();
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            error!("Failed to trigger sync: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}


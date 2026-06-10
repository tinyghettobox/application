use database::{
    model::sync_config::{CreateSyncConfig, SortBy},
    DatabaseConnection, SyncConfigRepository, SyncStatusRepository,
};
use sea_orm::{FromQueryResult, Statement};
use tracing::info;

/// Inserts default `sync_config` rows for any Spotify container entry that was
/// added before the sync system existed (i.e. has no `sync_config` row yet).
pub async fn run(conn: &DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    let already_configured = SyncConfigRepository::get_all_ids(conn).await?;

    let candidates = load_candidates(conn).await?;

    for row in candidates {
        if already_configured.contains(&row.library_entry_id) {
            continue;
        }

        let sort_by = default_sort_for(&row.spotify_type);
        let _ = SyncConfigRepository::upsert(
            conn,
            row.library_entry_id,
            CreateSyncConfig { sort_by, sort_regex: None, sync_interval_days: 30 },
        )
        .await;
        let _ = SyncStatusRepository::set_pending(conn, row.library_entry_id).await;

        info!(
            "Sync job migration: registered entry {} ({})",
            row.library_entry_id, row.spotify_type
        );
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

#[derive(FromQueryResult)]
struct LegacyEntry {
    library_entry_id: i32,
    spotify_type: String,
}

async fn load_candidates(conn: &DatabaseConnection) -> Result<Vec<LegacyEntry>, sea_orm::DbErr> {
    LegacyEntry::find_by_statement(Statement::from_sql_and_values(
        sea_orm::DbBackend::Sqlite,
        r#"
        SELECT ts.library_entry_id, ts.spotify_type
        FROM track_source ts
        JOIN library_entry le ON le.id = ts.library_entry_id
        WHERE ts.spotify_type IN ('artist', 'album', 'playlist', 'show')
          AND ts.library_entry_id IS NOT NULL
        "#,
        vec![],
    ))
    .all(conn)
    .await
}

fn default_sort_for(spotify_type: &str) -> SortBy {
    match spotify_type {
        "album" | "playlist" => SortBy::TrackNumberAsc,
        _ => SortBy::ReleaseDateDesc,
    }
}

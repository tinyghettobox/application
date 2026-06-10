use std::collections::{HashSet, HashMap};

use tracing::{debug, info, warn};
use database::{
    model::{
        library_entry::{CreateModel as LibraryEntryCreateModel, Variant},
        sync_config::SyncConfig,
        track_source::CreateModel as TrackSourceCreateModel,
    },
    DatabaseConnection, LibraryEntryRepository, SyncStatusRepository,
};
use rspotify::AuthCodeSpotify;
use sea_orm::{ConnectionTrait, Statement, Value};

use super::SyncError;
use super::sort;
use super::image;
use super::spotify_fetch::{fetch_all_children, is_container, SpotifyItem};

/// Entry point called from `spawn_blocking`. Syncs all children of a Spotify
/// container entry recursively (artist → albums → tracks, etc.) and updates
/// the DB with additions, deletions, and re-ordering.
pub fn sync_entry(
    conn: &DatabaseConnection,
    library_entry_id: i32,
    name: &str,
    spotify_id: &str,
    spotify_type: &str,
    config: &SyncConfig,
    spotify: &AuthCodeSpotify,
) -> Result<(), SyncError> {
    let rt = tokio::runtime::Handle::current();

    info!(
        entry_id = library_entry_id,
        name,
        spotify_type,
        spotify_id,
        "sync_entry: fetching children from Spotify"
    );
    let spotify_items = fetch_all_children(spotify_id, spotify_type, spotify)?;
    let total = spotify_items.len() as i32;
    info!(
        entry_id = library_entry_id,
        total,
        "sync_entry: fetched {} items from Spotify",
        total
    );

    rt.block_on(SyncStatusRepository::update_progress(conn, library_entry_id, 0, Some(total)))
        .ok();

    let existing = load_existing_children(conn, library_entry_id, &rt)?;
    debug!(
        entry_id = library_entry_id,
        existing_count = existing.len(),
        "sync_entry: loaded {} existing children from DB",
        existing.len()
    );

    let (seen, new_ids) = reconcile_children(conn, library_entry_id, &spotify_items, &existing, config, spotify, &rt)?;

    mark_removed_as_deleted(conn, &existing, &seen, &rt)?;

    sort::apply(conn, library_entry_id, &spotify_items, &new_ids, config, &rt)?;

    info!(entry_id = library_entry_id, name = name, "sync_entry: done");
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

type ExistingChild = (
    database::model::library_entry::Model,
    Option<database::model::track_source::Model>,
);

fn load_existing_children(
    conn: &DatabaseConnection,
    parent_id: i32,
    rt: &tokio::runtime::Handle,
) -> Result<Vec<ExistingChild>, SyncError> {
    use database::model::library_entry::{Column, Entity};
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

    rt.block_on(async {
        Entity::find()
            .filter(Column::ParentId.eq(parent_id))
            .find_also_related(database::model::track_source::Entity)
            .all(conn)
            .await
    })
    .map_err(|e| SyncError::Database(e.to_string()))
}

/// Processes each Spotify item: un-deletes existing ones, creates new ones.
/// Returns the set of Spotify IDs seen in this sync pass.
fn reconcile_children(
    conn: &DatabaseConnection,
    parent_id: i32,
    spotify_items: &[SpotifyItem],
    existing: &[ExistingChild],
    config: &SyncConfig,
    spotify: &AuthCodeSpotify,
    rt: &tokio::runtime::Handle,
) -> Result<(HashSet<String>, HashMap<String, i32>), SyncError> {
    let mut seen: HashSet<String> = HashSet::new();
    // Maps spotify_id → newly created library_entry id
    let mut new_ids: HashMap<String, i32> = HashMap::new();
    let mut items_done = 0i32;
    let total = spotify_items.len() as i32;

    for item in spotify_items {
        seen.insert(item.spotify_id.clone());

        let matched = existing.iter().find(|(_, ts)| {
            ts.as_ref()
                .and_then(|t| t.spotify_id.as_ref())
                .map(|id| id == &item.spotify_id)
                .unwrap_or(false)
        });

        if let Some((entry, track_source)) = matched {
            if entry.deleted {
                info!(entry_id = entry.id, name = %entry.name, "undeleting entry that reappeared on Spotify");
                undelete_entry(conn, entry.id, rt)?;
            } else {
                debug!(entry_id = entry.id, name = %entry.name, "existing entry up-to-date, skipping create");
            }
            if let Some(ts) = track_source {
                if let (Some(cid), Some(ctype)) = (&ts.spotify_id, &ts.spotify_type) {
                    if is_container(ctype) {
                        debug!(entry_id = entry.id, spotify_type = %ctype, "recursing into container");
                        sync_entry(conn, entry.id, &entry.name, cid, ctype, config, spotify)?;
                    }
                }
            }
        } else {
            info!(spotify_id = %item.spotify_id, name = %item.name, "creating new child entry");
            let new_id = create_child(conn, parent_id, item, existing.len(), config, spotify, rt)?;
            new_ids.insert(item.spotify_id.clone(), new_id);
        }

        items_done += 1;
        rt.block_on(SyncStatusRepository::update_progress(conn, parent_id, items_done, Some(total)))
            .ok();
    }

    Ok((seen, new_ids))
}

fn undelete_entry(
    conn: &DatabaseConnection,
    id: i32,
    rt: &tokio::runtime::Handle,
) -> Result<(), SyncError> {
    rt.block_on(async {
        conn.execute(Statement::from_sql_and_values(
            sea_orm::DbBackend::Sqlite,
            "UPDATE library_entry SET deleted = 0 WHERE id = ?",
            vec![Value::Int(Some(id))],
        ))
        .await
    })
    .map_err(|e| SyncError::Database(e.to_string()))?;
    Ok(())
}

fn create_child(
    conn: &DatabaseConnection,
    parent_id: i32,
    item: &SpotifyItem,
    existing_count: usize,
    config: &SyncConfig,
    spotify: &AuthCodeSpotify,
    rt: &tokio::runtime::Handle,
) -> Result<i32, SyncError> {
    let child_type = item.spotify_type.as_str();
    let cover_image = item.image_url.as_ref().and_then(|url| {
        image::fetch_and_resize(url).map_err(|e| warn!(url, error = %e, "failed to fetch cover image, skipping")).ok()
    });

    let create_model = LibraryEntryCreateModel {
        parent_id: Some(parent_id),
        variant: if is_container(child_type) { Variant::Folder } else { Variant::Spotify },
        name: item.name.clone(),
        image: cover_image,
        sort_key: existing_count as i32,
        children: None,
        track_source: Some(TrackSourceCreateModel {
            id: None,
            library_entry_id: None,
            title: item.name.clone(),
            url: None,
            file: None,
            spotify_id: Some(item.spotify_id.clone()),
            spotify_type: Some(child_type.to_string()),
        }),
    };

    let created = rt
        .block_on(LibraryEntryRepository::create(conn, Some(parent_id), vec![create_model]))
        .map_err(|e| SyncError::Database(e.to_string()))?;

    let new_entry = created.into_iter().next()
        .ok_or_else(|| SyncError::Database("create returned no entry".to_string()))?;

    if is_container(child_type) {
        sync_entry(conn, new_entry.id, &new_entry.name, &item.spotify_id, child_type, config, spotify)?;
    }

    Ok(new_entry.id)
}

fn mark_removed_as_deleted(
    conn: &DatabaseConnection,
    existing: &[ExistingChild],
    seen: &HashSet<String>,
    rt: &tokio::runtime::Handle,
) -> Result<(), SyncError> {
    for (entry, ts) in existing {
        let spotify_id = ts
            .as_ref()
            .and_then(|t| t.spotify_id.as_deref())
            .unwrap_or("");

        if !spotify_id.is_empty() && !seen.contains(spotify_id) {
            info!(entry_id = entry.id, name = %entry.name, spotify_id, "marking entry as deleted (no longer on Spotify)");
            rt.block_on(async {
                conn.execute(Statement::from_sql_and_values(
                    sea_orm::DbBackend::Sqlite,
                    "UPDATE library_entry SET deleted = 1 WHERE id = ?",
                    vec![Value::Int(Some(entry.id))],
                ))
                .await
            })
            .map_err(|e| SyncError::Database(e.to_string()))?;
        }
    }
    Ok(())
}

use std::cmp::Reverse;
use std::collections::HashMap;

use tracing::debug;
use database::{model::sync_config::SyncConfig, DatabaseConnection};
use sea_orm::{ConnectionTrait, FromQueryResult, Statement, Value};

use super::SyncError;
use super::spotify_fetch::SpotifyItem;

/// Updates `sort_key` for all non-deleted children of `parent_id`.
///
/// For existing items the current relative order (from drag-and-drop) is
/// preserved. New items (identified by `new_ids`: spotify_id → entry id) are
/// inserted at the position the configured sort strategy would place them
/// among the full set of children. Everything is renumbered 0, 1, 2, … so
/// sort_keys stay compact.
///
/// A `Manual` strategy appends new items at the end.
pub fn apply(
    conn: &DatabaseConnection,
    parent_id: i32,
    spotify_items: &[SpotifyItem],
    new_ids: &HashMap<String, i32>,
    config: &SyncConfig,
    rt: &tokio::runtime::Handle,
) -> Result<(), SyncError> {
    use database::model::sync_config::SortBy;

    let rows = load_child_rows(conn, parent_id, rt)?;
    if rows.is_empty() {
        return Ok(());
    }

    debug!(parent_id, sort_by = ?config.sort_by, rows = rows.len(), new = new_ids.len(), "applying sort");

    // Partition into existing (keep relative order) and new (to be inserted).
    // "existing" are sorted by their current sort_key so drag-and-drop order is
    // respected.
    let mut existing: Vec<&ChildRow> = rows.iter()
        .filter(|r| !new_ids.values().any(|&id| id == r.id))
        .collect();
    existing.sort_by_key(|r| r.sort_key);

    let new_rows: Vec<&ChildRow> = rows.iter()
        .filter(|r| new_ids.values().any(|&id| id == r.id))
        .collect();

    if new_rows.is_empty() {
        // Nothing new — no sort_keys need changing.
        return Ok(());
    }

    // For Manual strategy just append new items after existing ones.
    if matches!(config.sort_by, SortBy::Manual) {
        let mut result: Vec<&ChildRow> = existing;
        result.extend(new_rows);
        return write_sort_keys_indexed(conn, &result, rt);
    }

    // Build the ideal full ordering of ALL rows according to the strategy.
    // Map spotify_id → position in the Spotify API response.
    let api_order: HashMap<&str, usize> = spotify_items
        .iter()
        .enumerate()
        .map(|(i, item)| (item.spotify_id.as_str(), i))
        .collect();

    let mut all_ordered: Vec<&ChildRow> = rows.iter().collect();

    match &config.sort_by {
        SortBy::ReleaseDateDesc => {
            all_ordered.sort_by_key(|r| api_position(&api_order, r));
        }
        SortBy::ReleaseDateAsc => {
            all_ordered.sort_by_key(|r| Reverse(api_position(&api_order, r)));
        }
        SortBy::AlphaAsc => {
            all_ordered.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        }
        SortBy::AlphaDesc => {
            all_ordered.sort_by(|a, b| b.name.to_lowercase().cmp(&a.name.to_lowercase()));
        }
        SortBy::TrackNumberAsc => {
            all_ordered.sort_by_key(|r| api_position(&api_order, r));
        }
        SortBy::MostRecentlyPlayed => {
            all_ordered.sort_by(|a, b| b.played_at.cmp(&a.played_at));
        }
        SortBy::EpisodeRegexAsc | SortBy::EpisodeRegexDesc => {
            let pattern = config.sort_regex.as_deref().unwrap_or(r"(\d+)");
            let re = regex_lite::Regex::new(pattern).ok();
            all_ordered.sort_by(|a, b| {
                let na = extract_episode_number(re.as_ref(), &a.name);
                let nb = extract_episode_number(re.as_ref(), &b.name);
                if matches!(config.sort_by, SortBy::EpisodeRegexAsc) {
                    na.cmp(&nb)
                } else {
                    nb.cmp(&na)
                }
            });
        }
        SortBy::Manual => unreachable!(),
    }

    // Build a position map: entry id → ideal index in the full sorted order.
    // Used to guide the merge walk below.
    let _ideal_pos: HashMap<i32, usize> = all_ordered.iter()
        .enumerate()
        .map(|(pos, r)| (r.id, pos))
        .collect();

    // Merge: walk the ideal order; when we encounter a new item, emit it;
    // when we encounter an existing item slot, emit the next existing item in
    // its current (drag-and-drop) order. This preserves the relative order of
    // existing items while inserting new items at their strategy-determined
    // positions.
    let mut result: Vec<&ChildRow> = Vec::with_capacity(rows.len());
    let mut existing_iter = existing.iter().peekable();

    for row in &all_ordered {
        if new_ids.values().any(|&id| id == row.id) {
            // New item — insert it here.
            result.push(row);
        } else {
            // Existing item slot — emit the next existing item in drag order.
            if let Some(next_existing) = existing_iter.next() {
                result.push(next_existing);
            }
        }
    }
    // Drain any remaining existing items (shouldn't happen but be safe).
    for leftover in existing_iter {
        result.push(leftover);
    }

    // Sanity check: only write if row counts match.
    if result.len() != rows.len() {
        debug!(parent_id, "sort merge produced unexpected length, skipping write");
        return Ok(());
    }

    write_sort_keys_indexed(conn, &result, rt)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

#[derive(FromQueryResult)]
struct ChildRow {
    id: i32,
    name: String,
    sort_key: i32,
    played_at: Option<String>,
    spotify_id: Option<String>,
}

fn load_child_rows(
    conn: &DatabaseConnection,
    parent_id: i32,
    rt: &tokio::runtime::Handle,
) -> Result<Vec<ChildRow>, SyncError> {
    rt.block_on(async {
        ChildRow::find_by_statement(Statement::from_sql_and_values(
            sea_orm::DbBackend::Sqlite,
            r#"
            SELECT le.id, le.name, le.sort_key, le.played_at, ts.spotify_id
            FROM library_entry le
            LEFT JOIN track_source ts ON ts.library_entry_id = le.id
            WHERE le.parent_id = ? AND le.deleted = 0
            "#,
            vec![Value::Int(Some(parent_id))],
        ))
        .all(conn)
        .await
    })
    .map_err(|e| SyncError::Database(e.to_string()))
}

/// Writes sort_keys 0, 1, 2, … for the given ordered slice, skipping rows
/// whose sort_key is already correct.
fn write_sort_keys_indexed(
    conn: &DatabaseConnection,
    ordered: &[&ChildRow],
    rt: &tokio::runtime::Handle,
) -> Result<(), SyncError> {
    for (new_key, row) in ordered.iter().enumerate() {
        if row.sort_key == new_key as i32 {
            continue;
        }
        rt.block_on(async {
            conn.execute(Statement::from_sql_and_values(
                sea_orm::DbBackend::Sqlite,
                "UPDATE library_entry SET sort_key = ? WHERE id = ?",
                vec![Value::Int(Some(new_key as i32)), Value::Int(Some(row.id))],
            ))
            .await
        })
        .map_err(|e| SyncError::Database(e.to_string()))?;
    }
    Ok(())
}

fn api_position(order: &std::collections::HashMap<&str, usize>, row: &ChildRow) -> usize {
    row.spotify_id
        .as_deref()
        .and_then(|id| order.get(id))
        .copied()
        .unwrap_or(usize::MAX)
}

fn extract_episode_number(re: Option<&regex_lite::Regex>, name: &str) -> u64 {
    re.and_then(|r| r.captures(name))
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse().ok())
        .unwrap_or(u64::MAX)
}

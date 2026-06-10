mod image;
mod migrate;
mod runner;
mod sort;
mod spotify_fetch;

use database::{DatabaseConnection, SyncConfigRepository, SyncStatusRepository};
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::time::{sleep, Duration};
use tracing::{info, warn};

/// Seconds to wait between sync cycles under normal operation.
const NORMAL_INTERVAL_SECS: u64 = 5;
/// Initial startup delay before the first cycle.
const STARTUP_DELAY_SECS: u64 = 60;
/// Minimum extra backoff added on top of the normal interval when rate-limited.
const RATE_LIMIT_MIN_BACKOFF_SECS: u64 = 60;

/// Categorised error returned from a blocking sync operation.
#[derive(Debug)]
pub(super) enum SyncError {
    /// Spotify returned HTTP 429. Value is seconds to wait before retrying.
    RateLimited(u64),
    Spotify(String),
    Database(String),
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncError::RateLimited(s) => write!(f, "Rate limited â€“ retry in {}s", s),
            SyncError::Spotify(m) => write!(f, "Spotify error: {}", m),
            SyncError::Database(m) => write!(f, "Database error: {}", m),
        }
    }
}

enum CycleResult {
    NothingToDo,
    Done,
    RateLimited(u64),
    Error(String),
}

/// Spawns the background sync task. Returns immediately; the task runs for the
/// lifetime of the process. `notify` is signalled by route handlers when a
/// sync is triggered so the job wakes up immediately instead of waiting the
/// normal 5-minute interval.
pub fn start(conn: DatabaseConnection, notify: Arc<Notify>) {
    tokio::spawn(async move {
        info!("Sync job: waiting {}s before first cycle", STARTUP_DELAY_SECS);
        tokio::select! {
            _ = sleep(Duration::from_secs(STARTUP_DELAY_SECS)) => {}
            _ = notify.notified() => {
                info!("Sync job: woken during startup delay, starting first cycle immediately");
            }
        }

        if let Err(e) = migrate::run(&conn).await {
            warn!("Sync job: legacy migration failed: {}", e);
        }

        match SyncStatusRepository::reset_stale_running(&conn).await {
            Ok(0) => {}
            Ok(n) => warn!("Sync job: reset {} stale 'running' entries to 'pending' after crash", n),
            Err(e) => warn!("Sync job: failed to reset stale running entries: {}", e),
        }

        let mut extra_wait_secs: u64 = 0;
        loop {
            if extra_wait_secs > 0 {
                info!("Sync job: waiting extra {}s (rate-limit backoff)", extra_wait_secs);
                tokio::select! {
                    _ = sleep(Duration::from_secs(extra_wait_secs)) => {}
                    _ = notify.notified() => {
                        info!("Sync job: woken during rate-limit backoff");
                    }
                }
                extra_wait_secs = 0;
            }

            // Process one entry at a time, sleeping the normal interval between each
            // to avoid hammering the Spotify API.
            loop {
                match run_cycle(&conn).await {
                    CycleResult::NothingToDo => {
                        break;
                    }
                    CycleResult::Done => {
                        info!("Sync job: cycle complete, sleeping {}s before next entry", NORMAL_INTERVAL_SECS);
                        // Wait the normal interval before picking the next entry, but allow a
                        // manual trigger to skip the wait so newly-added entries aren't blocked.
                        tokio::select! {
                            _ = sleep(Duration::from_secs(NORMAL_INTERVAL_SECS)) => {}
                            _ = notify.notified() => {
                                info!("Sync job: woken by trigger, running cycle immediately");
                            }
                        }
                    }
                    CycleResult::RateLimited(secs) => {
                        extra_wait_secs = secs;
                        info!("Sync job: rate limited, extra {}s after normal interval", secs);
                        break;
                    }
                    CycleResult::Error(msg) => {
                        warn!("Sync job: cycle error: {}", msg);
                        break;
                    }
                }
            }

            // Sleep for the normal interval but wake early if a sync is triggered.
            tokio::select! {
                _ = sleep(Duration::from_secs(NORMAL_INTERVAL_SECS)) => {}
                _ = notify.notified() => {
                    info!("Sync job: woken by trigger, running cycle immediately");
                }
            }
        }
    });
}

async fn run_cycle(conn: &DatabaseConnection) -> CycleResult {
    let next = match SyncConfigRepository::find_next_due(conn).await {
        Ok(Some(entry)) => entry,
        Ok(None) => return CycleResult::NothingToDo,
        Err(e) => return CycleResult::Error(format!("find_next_due: {}", e)),
    };

    let (library_entry_id, spotify_id, spotify_type, name, sync_config) = next;
    info!(
        "Sync job: syncing entry {} '{}' ({} id={})",
        library_entry_id, name, spotify_type, spotify_id
    );

    if let Err(e) = SyncStatusRepository::set_running(conn, library_entry_id).await {
        warn!("Sync job: could not set status=running for {}: {}", library_entry_id, e);
    }

    let spotify = match crate::spotify_client::get_spotify(conn).await.map_err(|e| e.to_string()) {
        Ok(s) => s,
        Err(msg) => {
            let full_msg = format!("could not get Spotify client: {}", msg);
            let _ = SyncStatusRepository::set_error(conn, library_entry_id, &full_msg).await;
            return CycleResult::Error(full_msg);
        }
    };

    let conn2 = conn.clone();
    let result = tokio::task::spawn_blocking(move || {
        runner::sync_entry(&conn2, library_entry_id, &name, &spotify_id, &spotify_type, &sync_config, &spotify)
    })
    .await;

    match result {
        Ok(Ok(())) => {
            let _ = SyncStatusRepository::set_done(conn, library_entry_id).await;
            let _ = SyncConfigRepository::update_last_synced_at(conn, library_entry_id).await;
            CycleResult::Done
        }
        Ok(Err(SyncError::RateLimited(secs))) => {
            let msg = format!("Rate limited â€“ retry in {}s", secs);
            let _ = SyncStatusRepository::set_error(conn, library_entry_id, &msg).await;
            CycleResult::RateLimited(secs + RATE_LIMIT_MIN_BACKOFF_SECS)
        }
        Ok(Err(e)) => {
            let msg = e.to_string();
            let _ = SyncStatusRepository::set_error(conn, library_entry_id, &msg).await;
            CycleResult::Error(msg)
        }
        Err(e) => {
            let msg = format!("spawn_blocking panicked: {}", e);
            let _ = SyncStatusRepository::set_error(conn, library_entry_id, &msg).await;
            CycleResult::Error(msg)
        }
    }
}

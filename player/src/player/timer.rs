use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tracing::error;

use database::model::library_entry::{Model as LibraryEntry, Variant};

use crate::{Player, Progress};

// Update progress position every second optimistically. FetchProgressTimer is used to correct the optimistic progress position
pub struct PlayerTimer;
impl PlayerTimer {
    pub fn start_progress_timer<P, T, E>(player: Arc<Mutex<Player<P, T, E>>>)
    where
        P: Fn(Progress) + 'static + Sync + Send,
        T: Fn(Option<LibraryEntry>) + 'static + Sync + Send,
        E: Fn(LibraryEntry) + 'static + Sync + Send,
    {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(1000));
            loop {
                interval.tick().await;

                let mut player = player.lock().await;
                let (variant, progress) = {
                    let mut current_track = player.current_track.lock().await;
                    let track = match current_track.as_mut() {
                        None => continue,
                        Some(current_track) if !current_track.playing => continue,
                        Some(current_track) => current_track,
                    };

                    track.progress.position += Duration::from_millis(1000);

                    if let Some(on_progress) = player.notify_progress.as_ref() {
                        on_progress(track.progress.clone())
                    }

                    (track.library_entry.variant, track.progress.clone())
                };

                // For spotify we want to add tracks to queue before they end to ensure seamless playing
                if matches!(variant, Variant::Spotify) {
                    if progress.position + Duration::from_secs(1) >= progress.duration {
                        if let Err(err) = player.on_track_end().await {
                            error!("Failed to play next track: {}", err);
                        }
                    }
                } else {
                    if progress.position >= progress.duration {
                        if let Err(err) = player.on_track_end().await {
                            error!("Failed to end track: {}", err);
                        }
                    }
                }
            }
        });
    }

    // Fetching progress is done in separate thread to not block progress update
    pub fn start_correct_progress_timer<P, T, E>(player: Arc<Mutex<Player<P, T, E>>>)
    where
        P: Fn(Progress) + 'static + Sync + Send,
        T: Fn(Option<LibraryEntry>) + 'static + Sync + Send,
        E: Fn(LibraryEntry) + 'static + Sync + Send,
    {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(5000));
            loop {
                interval.tick().await;
                let player = player.lock().await;

                let mut current_track = player.current_track.lock().await;
                let current_track = match current_track.as_mut() {
                    None => continue,
                    Some(track) if !track.playing => continue,
                    Some(track) => track,
                };

                let progress = match current_track.target.lock().await.get_progress().await {
                    Ok(progress) => progress,
                    Err(error) => {
                        error!("Could not fetch progress: {}", error);
                        continue;
                    }
                };

                current_track.progress = progress;

                if let Some(on_progress) = player.notify_progress.as_ref() {
                    on_progress(current_track.progress.clone())
                }
            }
        });
    }
}

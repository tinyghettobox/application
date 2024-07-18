use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{debug, error, info};
use database::model::library_entry::{Variant, Model as LibraryEntry};
use crate::{Player, Progress};

// Update progress position every second optimistically. FetchProgressTimer is used to correct the optimistic progress position
pub struct PlayerTimer;
impl PlayerTimer {
    pub fn start_progress_timer<P, T>(player: Arc<Mutex<Player<P, T>>>)
    where
        P: Fn(Progress) + 'static + Sync + Send,
        T: Fn(Option<LibraryEntry>) + 'static + Sync + Send
    {
        tokio::spawn(async move {
            info!("timer spawn running in {:?}", std::thread::current().id());
            let mut interval = tokio::time::interval(Duration::from_millis(1000));
            loop {
                interval.tick().await;
                debug!("timer start");

                let track = {
                    debug!("player.lock()");
                    let player = player.lock().await;
                    debug!("current_track.lock()");
                    let mut current_track = player.current_track.lock().await;
                    debug!("locked");
                    let track = match current_track.as_mut() {
                        None => continue,
                        Some(current_track) if !current_track.playing => continue,
                        Some(current_track) => current_track,
                    };

                    track.progress.position += Duration::from_millis(1000);
                    player.on_progress.as_ref().inspect(|on_progress| on_progress(track.progress.clone()));

                    track.clone()
                };
                debug!("timer within");

                debug!("player.lock()");
                let mut player = player.lock().await;
                debug!("locked");
                // For spotify we want to add tracks to queue before they end to ensure seamless playing
                if matches!(track.library_entry.variant, Variant::Spotify) {
                    if track.progress.position + Duration::from_secs(1) >= track.progress.duration {
                        if let Err(err) = player.on_track_end().await {
                            error!("Failed to play next track: {}", err);
                        }
                    }
                } else {
                    if track.progress.position >= track.progress.duration {
                        if let Err(err) = player.on_track_end().await {
                            error!("Failed to end track: {}", err);
                        }
                    }
                }
                debug!("timer end");
            }
        });
    }

    // Fetching is progress is done in separate thread to not block progress update
    pub fn start_correct_progress_timer<P, T>(player: Arc<Mutex<Player<P, T>>>)
    where
        P: Fn(Progress) + 'static + Sync + Send,
        T: Fn(Option<LibraryEntry>) + 'static + Sync + Send
    {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(5000));
            loop {
                interval.tick().await;
                debug!("timer correction start");
                debug!("player.lock()");
                let player = player.lock().await;

                debug!("current_track.lock()");
                let mut current_track  = player.current_track.lock().await;
                debug!("locked");
                let current_track = match current_track.as_mut() {
                    None => {
                        debug!("release locks");
                        continue
                    },
                    Some(track) if !track.playing => {
                        debug!("release locks");
                        continue
                    },
                    Some(track) => track,
                };

                debug!("target.lock()");
                let progress = match current_track.target.lock().await.get_progress().await {
                    Ok(progress) => progress,
                    Err(error) => {
                        error!("Could not fetch progress: {}", error);
                        continue;
                    }
                };

                current_track.progress = progress;
                player.on_progress.as_ref().inspect(|on_progress| on_progress(current_track.progress.clone()));

                debug!("timer correction end");
            }
        });
    }
}

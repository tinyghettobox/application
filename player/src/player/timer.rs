use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tracing::error;

use database::model::library_entry::{Model as LibraryEntry, Variant};

use crate::player::play_target::ProgressStatus;
use crate::{Player, Progress};

// Update progress position every second optimistically. FetchProgressTimer is used to correct the optimistic progress position
pub struct PlayerTimer;
impl PlayerTimer {
    pub fn start_progress_timer<P, T, E, F>(_player: Arc<Mutex<Player<P, T, E, F>>>)
    where
        P: Fn(Progress) + 'static + Sync + Send,
        T: Fn(Option<LibraryEntry>) + 'static + Sync + Send,
        E: Fn(LibraryEntry) + 'static + Sync + Send,
        F: Fn(String) + 'static + Sync + Send,
    {
        // tokio::spawn(async move {
        //     let mut interval = tokio::time::interval(Duration::from_millis(1000));
        //     let mut last_update = Instant::now();
        //     loop {
        //         interval.tick().await;
        //
        //         let mut player = player.lock().await;
        //         let (variant, progress) = {
        //             let mut current_track = player.current_track.lock().await;
        //             let track = match current_track.as_mut() {
        //                 None => continue,
        //                 Some(current_track) if !current_track.playing => continue,
        //                 Some(current_track) => current_track,
        //             };
        //
        //             let now = Instant::now();
        //             track.progress.position += now.duration_since(last_update);
        //             last_update = now;
        //
        //             if let Some(on_progress) = player.notify_progress.as_ref() {
        //                 on_progress(track.progress.clone())
        //             }
        //
        //             (track.library_entry.variant, track.progress.clone())
        //         };
        //         // For infinite streams there is no track end so we skip that part
        //         if !progress.is_finite {
        //             continue;
        //         }
        //
        //         // For spotify we want to add tracks to queue before they end to ensure seamless playing
        //         if matches!(variant, Variant::Spotify) {
        //             if progress.position >= progress.duration {
        //                 if let Err(err) = player.on_track_end().await {
        //                     error!("Failed to play next track: {}", err);
        //                 }
        //             }
        //         } else {
        //             if progress.position >= progress.duration {
        //                 if let Err(err) = player.on_track_end().await {
        //                     error!("Failed to end track: {}", err);
        //                 }
        //             }
        //         }
        //     }
        // });
    }

    // Fetching progress is done in separate thread to not block progress update
    pub fn start_correct_progress_timer<P, T, E, F>(player: Arc<Mutex<Player<P, T, E, F>>>)
    where
        P: Fn(Progress) + 'static + Sync + Send,
        T: Fn(Option<LibraryEntry>) + 'static + Sync + Send,
        E: Fn(LibraryEntry) + 'static + Sync + Send,
        F: Fn(String) + 'static + Sync + Send,
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
                        Some(track) if !track.playing => continue,
                        Some(track) => track,
                    };

                    let progress = match track.target.lock().await.get_progress().await {
                        Ok(progress) => progress,
                        Err(error) => {
                            error!("Could not fetch progress: {}", error);
                            continue;
                        }
                    };

                    track.progress = progress.clone();

                    if let Some(on_progress) = player.notify_progress.as_ref() {
                        on_progress(track.progress.clone())
                    }

                    match progress.status {
                        ProgressStatus::Failed(reason) => {
                            if let Some(on_error) = player.notify_error.as_ref() {
                                on_error(reason.to_owned());
                            }
                            track.playing = false;
                            continue;
                        }
                        ProgressStatus::Stopped if progress.is_finite => {
                            // Kira signals natural end via Stopped; mark as done to prevent re-entry.
                            track.playing = false;
                        }
                        _ => {}
                    }

                    (track.library_entry.variant, progress.clone())
                };

                // For infinite streams there is no track end so we skip that part
                if !progress.is_finite {
                    continue;
                }

                // For spotify we want to add tracks to queue before they end to ensure seamless playing

                if progress.position >= progress.duration || matches!(progress.status, ProgressStatus::Stopped) {
                    // Mark track as not playing before on_track_end to prevent re-entry on next tick.
                    if let Some(track) = player.current_track.lock().await.as_mut() {
                        track.playing = false;
                    }
                    if let Err(err) = player.on_track_end().await {
                        error!("Failed to end track: {}", err);
                    }
                } else if matches!(variant, Variant::Spotify) {
                    if !progress.preloaded && progress.position > Duration::from_secs(10) {
                        if let Err(err) = player.queue_next_track().await {
                            error!("Failed to play next track: {}", err);
                        }
                    }
                }
            }
        });
    }
}

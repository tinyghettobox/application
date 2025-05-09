use chrono::Utc;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::Mutex as AsyncMutex;
use tracing::{debug, error, info, warn};

use database::model::library_entry::Variant;
use database::{model::library_entry::Model as LibraryEntry, LibraryEntryRepository, SystemConfigRepository};
use player::{Player, Progress};

use crate::state::{Dispatcher, State};

#[derive(Debug)]
pub enum Action {
    Started,
    Select(i32),
    Play(i32, Option<i32>), // Parent id and start id
    TogglePlay,
    NextTrack,
    PrevTrack,
    SetPlayedAt,
    Seek(f64),
    SetProgress(f64), // 0-1
    SetPlayingTrack(Option<LibraryEntry>),
    SetVolume(f64),
    ToggleMonitor(bool),
    Shutdown,
    TrackActivity,
}

#[derive(Debug)]
pub enum Event {
    LibraryEntryChanged,
    PlayStateChanged,
    ProgressChanged,
    TrackPlayed,
    TrackChanged,
    VolumeChanged,
    MonitorToggled,
    Error(String),
    Dummy,
}

pub trait EventHandler {
    fn on_event(&mut self, event: &Event);
    fn get_children(&self) -> Vec<Arc<Mutex<Box<dyn EventHandler>>>>;
}

impl Action {
    pub async fn process<P, T, E>(
        action: Action,
        state: Arc<Mutex<State>>,
        dispatcher: Arc<Mutex<Dispatcher>>,
        player: Arc<AsyncMutex<Player<P, T, E>>>,
    ) where
        P: Fn(Progress) + 'static + Sync + Send,
        T: Fn(Option<LibraryEntry>) + 'static + Sync + Send,
        E: Fn(LibraryEntry) + 'static + Sync + Send,
    {
        match action {
            Action::Started => {
                state.lock().unwrap().started = true;
            }
            Action::Select(library_entry_id) => {
                let connection = state.lock().unwrap().connection.clone();
                let library_entry =
                    LibraryEntryRepository::get(&connection, library_entry_id).await.unwrap_or_else(|error| {
                        error!("Could not load library entry '{}': {}", library_entry_id, error);
                        None
                    });

                match library_entry {
                    None => {
                        error!("No library entry '{}' found", library_entry_id);
                    }
                    Some(library_entry) => {
                        let variants = library_entry
                            .children
                            .as_ref()
                            .map(|children| children.iter().map(|entry| entry.variant).collect::<Vec<Variant>>());

                        let mut state = state.lock().unwrap();
                        match variants {
                            Some(variants) => {
                                if variants.len() == 0 {
                                    state.active_view = "empty_info".to_string();
                                } else if variants.contains(&Variant::Folder) || variants.contains(&Variant::Stream) {
                                    state.active_view = "tile_list".to_string();
                                } else {
                                    state.active_view = "detail_list".to_string();
                                }
                            }
                            None => {
                                state.active_view = "empty_info".to_string();
                            }
                        }
                        state.library_entry = library_entry;

                        dispatcher.lock().unwrap().dispatch_event(Event::LibraryEntryChanged);
                    }
                }
            }
            Action::Play(parent_id, start_id) => {
                let connection = state.lock().unwrap().connection.clone();
                let event = match LibraryEntryRepository::get_tracks_in_parent(&connection, parent_id).await {
                    Ok(library_entries) => {
                        let queue = if let Some(start_id) = start_id {
                            library_entries.into_iter().skip_while(|entry| entry.id != start_id).collect()
                        } else {
                            library_entries.into_iter().collect()
                        };
                        debug!("playing queue: {}", queue);

                        match player.lock().await.play_queue(queue).await {
                            Ok(Some(_)) => {
                                // Handled by on_track_change triggering SetPlayingTrack
                                None
                            }
                            Ok(None) => Some(Event::Error("Did not play anything".to_string())),
                            Err(error) => Some(Event::Error(format!("Could not play: {}", error))),
                        }
                    }
                    Err(error) => Some(Event::Error(format!("Could not play: {}", error))),
                };

                if let Some(event) = event {
                    dispatcher.lock().unwrap().dispatch_event(event);
                }
            }
            Action::SetPlayedAt => {
                let library_entry =
                    {
                        let mut state = state.lock().unwrap();
                        let library_entry_id = state.playing_library_entry.as_ref().map(|entry| entry.id).unwrap_or(-1);
                        state.library_entry.children.as_mut().and_then(|children| {
                            children.iter_mut().find(|c| c.id == library_entry_id).map(|c| c.clone())
                        })
                    };

                if let Some(mut entry) = library_entry {
                    entry.played_at = Some(chrono::Utc::now());
                    let connection = state.lock().unwrap().connection.clone();
                    match LibraryEntryRepository::mark_played(&connection, entry.id, entry.played_at).await {
                        Ok(_) => {
                            dispatcher.lock().unwrap().dispatch_event(Event::TrackPlayed);
                        }
                        Err(error) => error!("Could not mark library entry as played: {}", error),
                    }
                } else {
                    warn!("Could not find library entry to mark as played")
                }
            }
            Action::SetPlayingTrack(library_entry) => {
                let mut state = state.lock().unwrap();
                state.playing_library_entry = library_entry.clone();
                state.paused = library_entry.is_none();
                state.progress = 0.0;

                dispatcher.lock().unwrap().dispatch_event(Event::TrackChanged);
            }
            Action::TogglePlay => {
                let result = if state.lock().unwrap().paused {
                    player.lock().await.resume().await
                } else {
                    player.lock().await.pause().await
                };
                let event = match result {
                    Ok(_) => {
                        let mut state = state.lock().unwrap();
                        state.paused = !state.paused;
                        Event::PlayStateChanged
                    }
                    Err(error) => Event::Error(error),
                };
                dispatcher.lock().unwrap().dispatch_event(event);
            }
            Action::NextTrack | Action::PrevTrack => {
                let mut player = player.lock().await;
                let result = if matches!(action, Action::NextTrack) {
                    player.play_next_track().await
                } else {
                    player.play_prev_track().await
                };
                let event = match result {
                    Ok(new_track) => {
                        let mut state = state.lock().unwrap();
                        match new_track {
                            Some(track) => {
                                state.playing_library_entry = Some(track.clone());
                                state.paused = false;
                                state.progress = 0.0;
                                Event::TrackChanged
                            }
                            None => {
                                state.playing_library_entry = None;
                                state.paused = true;
                                state.progress = 0.0;
                                Event::PlayStateChanged
                            }
                        }
                    }
                    Err(error) => Event::Error(error),
                };
                dispatcher.lock().unwrap().dispatch_event(event);
            }
            Action::SetProgress(progress) => {
                state.lock().unwrap().progress = progress;
                dispatcher.lock().unwrap().dispatch_event(Event::ProgressChanged);
            }
            Action::Seek(percent) => {
                let mut player_guard = player.lock().await;
                let event = match player_guard.seek_to(percent).await {
                    Ok(Some(progress)) => {
                        state.lock().unwrap().progress = progress.as_f64();
                        Event::ProgressChanged
                    }
                    Ok(None) => Event::ProgressChanged,
                    Err(error) => Event::Error(format!("Could not seek: {}", error)),
                };

                dispatcher.lock().unwrap().dispatch_event(event);
            }
            Action::SetVolume(volume) => {
                // Delay setting the volume during startup
                if !state.lock().unwrap().started {
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
                let event = match player.lock().await.set_volume(volume).await {
                    Ok(_) => {
                        let connection = state.lock().unwrap().connection.clone();
                        match SystemConfigRepository::set_volume(&connection, (volume * 100.0) as u8).await {
                            Ok(_) => {
                                state.lock().unwrap().volume = volume;
                                Event::VolumeChanged
                            }
                            Err(error) => Event::Error(format!("Could not save volume to database: {}", error)),
                        }
                    }
                    Err(error) => Event::Error(format!("Could not change volume: {}", error)),
                };

                dispatcher.lock().unwrap().dispatch_event(event);
            }
            Action::ToggleMonitor(active) => {
                let mut state = state.lock().expect("could not lock");

                if cfg!(target_arch = "arm") {
                    info!("Toggling display");
                    let result = Command::new("vcgencmd").arg("display_power").arg((active as i32).to_string()).spawn();

                    if result.is_err() {
                        error!("Could not toggle display: {:?}", result);
                    }
                }

                state.monitor_active = active;
                dispatcher.lock().unwrap().dispatch_event(Event::MonitorToggled);
            }
            Action::Shutdown => {
                if cfg!(target_arch = "arm") {
                    info!("Shutting down");
                    Command::new("shutdown").arg("now").spawn().expect("could not shutdown");
                }
            }
            Action::TrackActivity => {
                let mut state = state.lock().expect("could not lock");
                state.last_activity = Utc::now().timestamp();
            }
        }
    }
}

impl Event {
    pub fn broadcast(event: Event, listener: Arc<Mutex<Box<dyn EventHandler>>>) {
        debug!("Handling event {:?}", event);
        let mut listeners = vec![listener.clone()];
        while let Some(listener) = listeners.pop() {
            let mut listener = listener.lock().unwrap();
            for child in listener.get_children() {
                listeners.push(child);
            }
            listener.on_event(&event);
        }
    }
}

use std::sync::{Arc};
use std::sync::Mutex;
use gtk4::glib;
use tokio::sync::Mutex as AsyncMutex;
use tracing::{debug, error, info};
use database::{LibraryEntryRepository, model::library_entry::Model as LibraryEntry};
use database::model::library_entry::Variant;
use player::{Player, Progress};
use crate::state::{Dispatcher, State};

#[derive(Debug)]
pub enum Action {
    Select(i32),
    SetLibraryEntry(LibraryEntry),
    Play(LibraryEntry),
    TogglePlay,
    NextTrack,
    PrevTrack,
    Seek(f64),
    SetProgress(f64), // 0-100%
    SetPlayingTrack(Option<i32>),
    SetVolume(i32),
}

#[derive(Debug)]
pub enum Event {
    LibraryEntryChanged,
    PlayStateChanged,
    ProgressChanged,
    TrackChanged,
    VolumeChanged,
    Error(String),
}

pub trait EventHandler {
    fn on_event(&mut self, event: &Event);
    fn get_children(&self) -> Vec<Arc<Mutex<Box<dyn EventHandler>>>>;
}

impl Action {
    pub fn process<P, T>(action: Action, state: Arc<Mutex<State>>, dispatcher: Arc<Mutex<Dispatcher>>, player: Arc<AsyncMutex<Player<P, T>>>)
    where
        P: Fn(Progress) + 'static + Sync + Send,
        T: Fn(Option<LibraryEntry>) + 'static + Sync + Send
    {
        debug!("Handling action: {:?}", action);
        match action {
            Action::Select(library_entry_id) => {
                let state = state.clone();
                glib::MainContext::default().spawn_local(async move {

                    info!("select action spawn_local running in {:?}", std::thread::current().id());
                    debug!("select start");
                    let state = state.lock().expect("Could not lock state");
                    match LibraryEntryRepository::get(&state.connection, library_entry_id).await {
                        Ok(entry) => {
                            match entry {
                                Some(library_entry) => {
                                    dispatcher
                                        .lock()
                                        .expect("Could not lock dispatcher")
                                        .dispatch_action(Action::SetLibraryEntry(library_entry));
                                }
                                None => {
                                    error!("No library entry '{}' found", library_entry_id);
                                }
                            }
                        }
                        Err(error) => {
                            error!("Could not load library entry '{}': {}", library_entry_id, error);
                        }
                    }

                    debug!("select end");
                });
            }
            Action::SetLibraryEntry(library_entry) => {
                debug!("set library start");
                let mut state = state.lock().expect("Could not lock state");
                let variants = library_entry
                    .children
                    .as_ref()
                    .map(|children|
                        children.iter().map(|entry| entry.variant).collect::<Vec<Variant>>()
                    );

                match variants {
                    Some(variants) => {
                        if variants.contains(&Variant::Folder) || variants.contains(&Variant::Stream) {
                            state.active_view = "tile_list".to_string();
                        } else {
                            state.active_view = "detail_list".to_string();
                        }
                    },
                    None => {
                        state.active_view = "empty_info".to_string();
                    }
                }
                state.library_entry = library_entry;

                dispatcher
                    .lock()
                    .expect("Could not lock dispatcher")
                    .dispatch_event(Event::LibraryEntryChanged);

                debug!("set library end");
            }
            Action::Play(library_entry) => {
                let state = state.clone();
                let player = player.clone();
                glib::MainContext::default().spawn(async move {
                    info!("Play action spawn running in {:?}", std::thread::current().id());
                    debug!("play start");
                    let connection = state.lock().expect("Could not lock state").connection.clone();
                    let parent_id = library_entry.parent_id.expect("There should be always a parent");
                    let event = match LibraryEntryRepository::get_tracks_in_parent(&connection, parent_id).await {
                        Ok(library_entries) => {
                            let queue = library_entries.into_iter().skip_while(|entry| entry.id != library_entry.id).collect();
                            debug!("playing queue: {}", queue);
                            match player.lock().await.play_queue(queue).await {
                                Ok(Some(playing_library_entry)) => {
                                    let mut state = state.lock().expect("Could not lock state");
                                    state.playing_library_entry = Some(playing_library_entry.clone());
                                    state.paused = false;
                                    state.progress = 0.0;
                                    Event::TrackChanged
                                }
                                Ok(None) => Event::Error("Did not play anything".to_string()),
                                Err(error) => Event::Error(format!("Could not play: {}", error))
                            }
                        }
                        Err(error) => Event::Error(format!("Could not play: {}", error))
                    };

                    dispatcher
                        .lock()
                        .expect("Could not lock dispatcher")
                        .dispatch_event(event);

                    debug!("play end");
                });
            }
            Action::TogglePlay => {
                let state = state.clone();
                let player = player.clone();
                glib::MainContext::default().spawn(async move {
                    debug!("toggle play start");
                    let result = if state.lock().expect("Could not lock state").paused {
                        player.lock().await.resume().await
                    } else {
                        player.lock().await.pause().await
                    };
                    let event = match result {
                        Ok(_) => {
                            let mut state = state.lock().expect("Could not lock state");
                            state.paused = !state.paused;
                            Event::PlayStateChanged
                        },
                        Err(error) => Event::Error(error)
                    };
                    dispatcher
                        .lock()
                        .expect("Could not lock dispatcher")
                        .dispatch_event(event);

                    debug!("toggle play end");
                });
            }
            Action::NextTrack | Action::PrevTrack => {
                let state = state.clone();
                let player = player.clone();
                glib::MainContext::default().spawn(async move {
                    debug!("next/prev track start");
                    let mut player = player.lock().await;
                    let result = if matches!(action, Action::NextTrack) {
                        player.play_next_track().await
                    } else {
                        player.play_prev_track().await
                    };
                    let event = match result {
                        Ok(new_track) => {
                            let mut state = state.lock().expect("Could not lock state");
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
                        },
                        Err(error) => Event::Error(error)
                    };
                    dispatcher
                        .lock()
                        .expect("Could not lock dispatcher")
                        .dispatch_event(event);

                    debug!("next/prev track end");
                });
            }
            Action::SetProgress(progress) => {
                glib::MainContext::default().spawn_local(async move {
                    debug!("set progress start");
                    let mut state = state.lock().expect("Could not lock state");
                    state.progress = progress;
                    dispatcher
                        .lock()
                        .expect("Could not lock dispatcher")
                        .dispatch_event(Event::ProgressChanged);
                    debug!("set progress end");
                });
            }
            Action::Seek(percent) => {
                glib::MainContext::default().spawn(async move {
                    debug!("seek start");
                    let event = match player.lock().await.seek_to(percent).await {
                        Ok(_) => Event::ProgressChanged,
                        Err(error) => Event::Error(format!("Could not seek: {}", error))
                    };

                    dispatcher
                        .lock()
                        .expect("Could not lock dispatcher")
                        .dispatch_event(event);

                    debug!("seek end");
                });
            },
            Action::SetVolume(_volume) => {}
            Action::SetPlayingTrack(library_entry_id) => {
                debug!("set playing track start");
                let mut state = state.lock().expect("Could not lock state");
                state.playing_library_entry = library_entry_id.and_then(|library_entry_id| {
                    state.library_entry.children.as_ref().and_then(|children| {
                        children.iter().find(|entry| entry.id == library_entry_id).cloned()
                    })
                });

                dispatcher
                    .lock()
                    .expect("Could not lock dispatcher")
                    .dispatch_event(Event::TrackChanged);
                debug!("set playing track end");
            }
        }
        debug!("Handled action");
    }
}

impl Event {
    pub fn broadcast(event: Event, listener: Arc<Mutex<Box<dyn EventHandler>>>) {
        debug!("broadcasting event {:?}", event);
        let mut listeners = vec![listener.clone()];
        while let Some(listener) = listeners.pop() {
            let mut listener = listener.lock().expect("could not lock event listener");
            for child in listener.get_children() {
                listeners.push(child);
            }
            listener.on_event(&event);
        }
        debug!("broadcasting event done");
    }
}
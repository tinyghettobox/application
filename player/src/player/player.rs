use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::debug;
use tracing::log::{info, error};

use database::model::library_entry::Variant;
use database::{model::library_entry::Model as LibraryEntry, DatabaseConnection};

use crate::player::play_target::{LocalPlayTarget, PlayTarget, Progress, RemotePlayTarget, SpotifyPlayTarget};
use crate::player::queue::Queue;
use crate::player::spotify_manager::SpotifyManager;
use crate::player::timer::PlayerTimer;

#[derive(Clone)]
pub(super) struct Track {
    pub(super) library_entry: LibraryEntry,
    pub(super) target: Arc<Mutex<dyn PlayTarget + Send + 'static>>,
    pub(super) playing: bool,
    pub(super) progress: Progress,
}

#[derive(Clone)]
pub struct Player<P, T, E>
where
    P: Fn(Progress) + 'static + Sync + Send,
    T: Fn(Option<LibraryEntry>) + 'static + Sync + Send,
    E: Fn(LibraryEntry) + 'static + Sync + Send,
{
    spotify: Arc<Mutex<SpotifyPlayTarget>>,
    local: Arc<Mutex<LocalPlayTarget>>,
    remote: Arc<Mutex<RemotePlayTarget>>,
    queue: Queue,
    pub(super) current_track: Arc<Mutex<Option<Track>>>,
    pub(super) notify_progress: Option<P>,
    pub(super) notify_track_change: Option<T>,
    pub(super) notify_track_end: Option<E>,
}

impl<P, T, E> Player<P, T, E>
where
    P: Fn(Progress) + 'static + Sync + Send,
    T: Fn(Option<LibraryEntry>) + 'static + Sync + Send,
    E: Fn(LibraryEntry) + 'static + Sync + Send,
{
    pub async fn new(conn: DatabaseConnection, volume: f64) -> Arc<Mutex<Self>> {
        let spotify_manager = SpotifyManager::new(&conn).await;

        let player = Arc::new(Mutex::new(Self {
            spotify: Arc::new(Mutex::new(SpotifyPlayTarget::new(spotify_manager, volume).await)),
            local: Arc::new(Mutex::new(LocalPlayTarget::new(conn.clone(), volume).await)),
            remote: Arc::new(Mutex::new(RemotePlayTarget::new(conn.clone(), volume))),
            queue: Queue::new(),
            current_track: Arc::new(Mutex::new(None)),
            notify_progress: Default::default(),
            notify_track_change: Default::default(),
            notify_track_end: Default::default(),
        }));

        PlayerTimer::start_progress_timer(player.clone());
        PlayerTimer::start_correct_progress_timer(player.clone());

        player
    }

    pub fn connect_progress_changed(&mut self, notify_progress: P) {
        self.notify_progress = Some(notify_progress);
    }

    pub fn connect_track_changed(&mut self, notify_track_change: T) {
        self.notify_track_change = Some(notify_track_change);
    }

    pub fn connect_track_ended(&mut self, notify_track_end: E) {
        self.notify_track_end = Some(notify_track_end);
    }

    pub async fn play_queue(&mut self, queue: Queue) -> Result<Option<LibraryEntry>, String> {
        self.queue = queue;
        self.play_next_track().await
    }

    fn get_play_target(&mut self, track: &LibraryEntry) -> Option<Arc<Mutex<dyn PlayTarget + Send>>> {
        match track.variant {
            Variant::Folder => None,
            Variant::Stream => Some(self.remote.clone()),
            Variant::File => Some(self.local.clone()),
            Variant::Spotify => Some(self.spotify.clone()),
        }
    }

    async fn play_track(&mut self, library_entry: LibraryEntry) -> Result<Option<LibraryEntry>, String> {
        if let Some(current_track) = self.current_track.lock().await.as_mut() {
            if current_track.playing {
                debug!("Stopping currently playing track");
                current_track.target.lock().await.stop().await?;
                current_track.playing = false;
            }
        }

        let mut new_track = self.get_play_target(&library_entry).map(|target| Track {
            library_entry: library_entry.clone(),
            target,
            playing: true,
            progress: Progress::default(),
        });

        if let Some(new_track) = new_track.as_mut() {
            new_track.target.lock().await.play(&new_track.library_entry).await.map_err(
                |error| {
                    error!("#### Failed to play track: {}", error);
                    error
                }
            )?;
            sleep(Duration::from_secs(1)).await; // Let spotify api catch up with playing
            new_track.progress = new_track.target.lock().await.get_progress().await?;
            new_track.progress.position = Duration::from_secs(0); // Spotify returns weird position
        }

        self.current_track = Arc::new(Mutex::new(new_track));

        if let Some(on_track_change) = self.notify_track_change.as_ref() {
            on_track_change(Some(library_entry.clone()));
        }
        if let Some(on_progress) = self.notify_progress.as_ref() {
            on_progress(Progress::default());
        }

        Ok(Some(library_entry.clone()))
    }

    pub async fn play_prev_track(&mut self) -> Result<Option<LibraryEntry>, String> {
        match self.queue.prev() {
            Some(library_entry) => self.play_track(library_entry).await,
            None => Ok(None),
        }
    }

    pub async fn play_next_track(&mut self) -> Result<Option<LibraryEntry>, String> {
        match self.queue.next() {
            Some(library_entry) => self.play_track(library_entry).await,
            None => Ok(None),
        }
    }

    pub async fn queue_next_track(&mut self) -> Result<Option<LibraryEntry>, String> {
        match self.queue.next() {
            Some(library_entry) => self.play_track(library_entry).await,
            None => Ok(None),
        }
    }

    pub async fn pause(&mut self) -> Result<(), String> {
        if let Some(track) = self.current_track.lock().await.as_mut() {
            if track.playing {
                track.target.lock().await.pause().await?;
                track.playing = false;
            }
        }
        Ok(())
    }

    pub async fn resume(&mut self) -> Result<(), String> {
        if let Some(track) = self.current_track.lock().await.as_mut() {
            if !track.playing {
                track.target.lock().await.resume().await?;
                track.playing = true;
            }
        }
        Ok(())
    }

    pub async fn seek_to(&mut self, percent: f64) -> Result<Option<Progress>, String> {
        if let Some(track) = self.current_track.lock().await.as_mut() {
            let position = Duration::from_secs_f64(track.progress.duration.as_secs_f64() * percent / 100.0);
            let mut target_lock = track.target.lock().await;
            target_lock.seek_to(position).await?;

            track.progress.position = position;

            return Ok(Some(track.progress.clone()));
        }

        Ok(None)
    }

    pub async fn set_volume(&mut self, volume: f64) -> Result<(), String> {
        if let Some(track) = self.current_track.lock().await.as_mut() {
            track.target.lock().await.set_volume(volume).await?;
        }
        Ok(())
    }

    pub(super) async fn on_track_end(&mut self) -> Result<(), String> {
        info!("Track ended");
        if let Some(track) = self.current_track.lock().await.as_mut() {
            if let Some(notify_track_end) = self.notify_track_end.as_mut() {
                notify_track_end(track.library_entry.clone())
            }
        }

        self.current_track = Arc::new(Mutex::new(None));
        // If no next track notify about that. Else notification happens in play track
        if self.play_next_track().await?.is_none() {
            if let Some(on_track_change) = self.notify_track_change.as_ref() {
                on_track_change(None);
            }
        }
        Ok(())
    }
}

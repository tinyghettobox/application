use std::sync::{Arc};
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{debug, error};
use tracing::log::info;
use database::{DatabaseConnection, LibraryEntryRepository, model::library_entry::Model as LibraryEntry, SystemConfigRepository};
use database::model::library_entry::Variant;
use crate::player::play_target::{LocalPlayTarget, PlayTarget, Progress, RemotePlayTarget, SpotifyPlayTarget};
use crate::player::queue::Queue;
use crate::player::spotify::SpotifyManager;
use crate::player::timer::PlayerTimer;


#[derive(Clone)]
pub(super) struct Track {
    pub(super) library_entry: LibraryEntry,
    pub(super) target: Arc<Mutex<dyn PlayTarget + Send + 'static>>,
    pub(super) playing: bool,
    pub(super) progress: Progress
}

#[derive(Clone)]
pub struct Player<P, T>
where
    P: Fn(Progress) + 'static + Sync + Send,
    T: Fn(Option<LibraryEntry>) + 'static + Sync + Send
{
    conn: DatabaseConnection,
    spotify: Arc<Mutex<SpotifyPlayTarget>>,
    local: Arc<Mutex<LocalPlayTarget>>,
    remote: Arc<Mutex<RemotePlayTarget>>,
    queue: Queue,
    pub(super) current_track: Arc<Mutex<Option<Track>>>,
    pub(super) on_progress: Option<P>,
    pub(super) on_track_change: Option<T>
}

impl<P, T> Player<P, T>
where
    P: Fn(Progress) + 'static + Sync + Send,
    T: Fn(Option<LibraryEntry>) + 'static + Sync + Send
{
    pub async fn new(conn: &DatabaseConnection) -> Arc<Mutex<Self>> {
        let spotify_manager = SpotifyManager::new(&conn).await;
        let volume = SystemConfigRepository::get_volume(&conn).await.unwrap_or(30) as f64 / 100.0;

        let player = Arc::new(Mutex::new(Self {
            conn: conn.clone(),
            spotify: Arc::new(Mutex::new(SpotifyPlayTarget::new(spotify_manager, volume).await)),
            local: Arc::new(Mutex::new(LocalPlayTarget::new(conn.clone(), volume).await)),
            remote: Arc::new(Mutex::new(RemotePlayTarget::new(conn.clone(), volume))),
            queue: Queue::new(),
            current_track: Arc::new(Mutex::new(None)),
            on_progress: Default::default(),
            on_track_change: Default::default()
        }));

        PlayerTimer::start_progress_timer(player.clone());
        PlayerTimer::start_correct_progress_timer(player.clone());

        player
    }

    pub fn connect_progress_change(&mut self, on_progress: P) {
        self.on_progress = Some(on_progress);
    }

    pub fn connect_track_change(&mut self, on_track_change: T) {
        self.on_track_change = Some(on_track_change);
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
        let mut new_track = self.get_play_target(&library_entry)
            .map(|target| Track {
                library_entry: library_entry.clone(),
                target,
                playing: true,
                progress: Progress::default()
            });

        if let Some(new_track) = new_track.as_mut() {
            new_track.target.lock().await.play(&new_track.library_entry).await?;
            new_track.progress = new_track.target.lock().await.get_progress().await?;
            new_track.progress.position = Duration::from_secs(0); // Spotify returns weird position
            debug!("Progress after play start: {:?}", new_track.progress);
        }
        debug!("Current track set to some: {:?}", new_track.is_some());

        self.current_track = Arc::new(Mutex::new(new_track));

        self.on_track_change.as_ref().inspect(|on_track_change| on_track_change(Some(library_entry.clone())));
        self.on_progress.as_ref().inspect(|on_progress| on_progress(Progress::default()));

        Ok(Some(library_entry.clone()))
    }

    pub async fn play_prev_track(&mut self) -> Result<Option<LibraryEntry>, String> {
        match self.queue.prev() {
            Some(library_entry) => self.play_track(library_entry).await,
            None => Ok(None)
        }
    }

    pub async fn play_next_track(&mut self) -> Result<Option<LibraryEntry>, String> {
        match self.queue.next() {
            Some(library_entry) => self.play_track(library_entry).await,
            None => Ok(None)
        }
    }

    pub async fn queue_next_track(&mut self) -> Result<Option<LibraryEntry>, String> {
        match self.queue.next() {
            Some(library_entry) => self.play_track(library_entry).await,
            None => Ok(None)
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

    pub async fn seek_to(&mut self, percent: f64) -> Result<(), String> {
        debug!("current_track.lock()");
        if let Some(track) = self.current_track.lock().await.as_mut() {
            let position = Duration::from_secs_f64(track.progress.duration.as_secs_f64() * percent / 100.0);

            debug!("target.lock()");
            track.target.lock().await.seek_to(position).await?;

            debug!("locked and seeked");
            track.progress.position = position;
        }

        debug!("release");

        Ok(())
    }

    pub async fn set_volume(&mut self, volume: f64) -> Result<(), String> {
        if let Some(track) = self.current_track.lock().await.as_mut() {
            track.target.lock().await.set_volume(volume).await?;
            SystemConfigRepository::set_volume(&self.conn, (volume * 100.0) as u8).await
                .map_err(|e| format!("Failed to set volume: {}", e))?;
        }
        Ok(())
    }

    pub(super) async fn on_track_end(&mut self) -> Result<(), String> {
        info!("Track ended");
        if let Some(track) = self.current_track.lock().await.as_mut() {
            if let Err(error) = LibraryEntryRepository::set_played_at(&self.conn, track.library_entry.id).await {
                error!("Could not set played_at: {}", error)
            }
        }

        self.current_track = Arc::new(Mutex::new(None));
        // If no next track notify about that. Else notification happens in play track
        if self.play_next_track().await?.is_none() {
            self.on_track_change.as_ref().inspect(|on_track_change| on_track_change(None));
        }
        Ok(())
    }
}

use std::sync::{
    mpsc::{channel, Sender},
    Arc, Mutex,
};
use tracing;

use async_trait::async_trait;
use database::{model::{library_entry::Model as LibraryEntry, system_config::Model as SystemConfig}, DatabaseConnection};
use player::{Player, Progress, Queue};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

use super::actions::{Action, AudioStatus, LogEntry, WifiStatus};
use crate::with_getters_setters;

pub(super) const LOG_RING_CAPACITY: usize = 300;

type Changes = Vec<Field>;

#[async_trait]
pub trait PlayerControl: Send {
    async fn play_queue(&mut self, queue: Queue) -> Result<Option<LibraryEntry>, String>;
    async fn pause(&mut self) -> Result<(), String>;
    async fn resume(&mut self) -> Result<(), String>;
    async fn seek_to(&mut self, secs: f64) -> Result<(), String>;
    async fn play_prev(&mut self) -> Result<Option<LibraryEntry>, String>;
    async fn play_next(&mut self) -> Result<Option<LibraryEntry>, String>;
    async fn set_volume(&mut self, volume: f64) -> Result<(), String>;
}

with_getters_setters! {
    pub struct InnerState {
        pub messages: Vec<String>,
        pub active_library_entry: Option<LibraryEntry>,
        pub playing_library_entry: Option<LibraryEntry>,
        pub playing_ancestor_ids: Vec<i32>,
        pub is_playing: bool,
        pub is_loading: bool,
        pub progress: Progress,
        pub volume: i32,
        pub log_entries: Vec<LogEntry>,
        pub show_logs: bool,
        pub display_active: bool,
        pub audio_status: AudioStatus,
        pub wifi_status: WifiStatus,
        pub system_config: Option<SystemConfig>,
    }

    pub struct State {
        subscribers: Arc<Mutex<Vec<Sender<Changes>>>>,
        action_tx: Arc<Mutex<UnboundedSender<Action>>>,
        pub(super) player: Arc<tokio::sync::Mutex<Option<Box<dyn PlayerControl>>>>,
        pub conn: DatabaseConnection,
    }
}

impl Default for InnerState {
    fn default() -> Self {
        Self {
            changes: Vec::new(),
            messages: Vec::new(),
            active_library_entry: None,
            playing_library_entry: None,
            playing_ancestor_ids: Vec::new(),
            is_playing: false,
            is_loading: false,
            progress: Progress::default(),
            volume: 70,
            log_entries: Vec::new(),
            show_logs: false,
            display_active: true,
            audio_status: AudioStatus::Initializing,
            wifi_status: WifiStatus::Initializing,
            system_config: None,
        }
    }
}

struct PlayerAdapter<P, T, E, F>
where
    P: Fn(Progress) + Send + Sync + 'static,
    T: Fn(Option<LibraryEntry>) + Send + Sync + 'static,
    E: Fn(LibraryEntry) + Send + Sync + 'static,
    F: Fn(String) + Send + Sync + 'static,
{
    inner: Arc<tokio::sync::Mutex<Player<P, T, E, F>>>,
}

#[async_trait]
impl<P, T, E, F> PlayerControl for PlayerAdapter<P, T, E, F>
where
    P: Fn(Progress) + Send + Sync + 'static,
    T: Fn(Option<LibraryEntry>) + Send + Sync + 'static,
    E: Fn(LibraryEntry) + Send + Sync + 'static,
    F: Fn(String) + Send + Sync + 'static,
{
    async fn play_queue(&mut self, queue: Queue) -> Result<Option<LibraryEntry>, String> {
        self.inner.lock().await.play_queue(queue).await
    }

    async fn pause(&mut self) -> Result<(), String> {
        self.inner.lock().await.pause().await
    }

    async fn resume(&mut self) -> Result<(), String> {
        self.inner.lock().await.resume().await
    }

    async fn seek_to(&mut self, secs: f64) -> Result<(), String> {
        self.inner.lock().await.seek_to(secs).await.map(|_| ())
    }

    async fn play_prev(&mut self) -> Result<Option<LibraryEntry>, String> {
        self.inner.lock().await.play_prev_track().await
    }

    async fn play_next(&mut self) -> Result<Option<LibraryEntry>, String> {
        self.inner.lock().await.play_next_track().await
    }

    async fn set_volume(&mut self, volume: f64) -> Result<(), String> {
        self.inner.lock().await.set_volume(volume).await
    }
}

impl State {
    pub fn new(conn: DatabaseConnection) -> Self {
        let (tx, mut rx) = unbounded_channel::<Action>();
        let subscribers = Arc::new(Mutex::new(Vec::new()));
        let player: Arc<tokio::sync::Mutex<Option<Box<dyn PlayerControl>>>> =
            Arc::new(tokio::sync::Mutex::new(None));

        let self_ = Self {
            inner: Arc::new(Mutex::new(InnerState::default())),
            subscribers,
            action_tx: Arc::new(Mutex::new(tx)),
            player,
            conn,
        };

        {
            let mut self_clone = self_.clone();
            tokio::spawn(async move {
                while let Some(action) = rx.recv().await {
                    if action.should_log() {
                        tracing::debug!("Action: {:?}", action);
                    }
                    match action {
                        Action::InitVolume => self_clone.init_volume().await,
                        Action::LoadSystemConfig => self_clone.load_system_config().await,
                        Action::LoadLibraryEntry(id) => self_clone.load_library_entry(id).await,
                        Action::StartPlayingLibraryEntry(library_entry) => {
                            self_clone.start_playing_library_entry(library_entry);
                        }
                        Action::PlayLibraryEntry(library_entry) => {
                            self_clone.play_library_entry(library_entry).await
                        }
                        Action::TogglePlay(is_playing) => {
                            self_clone.toggle_play(is_playing).await
                        }
                        Action::SetProgress(progress) => self_clone.set_progress(progress),
                        Action::SetPlayingTrack(entry) => self_clone.set_playing_track(entry).await,
                        Action::SeekTo(pct) => self_clone.seek_to(pct).await,
                        Action::SetPlayedAt(id, played) => self_clone.set_played_at(id, played).await,
                        Action::PlayPrev => self_clone.play_prev().await,
                        Action::PlayNext => self_clone.play_next().await,
                        Action::SetVolume(v) => self_clone.set_volume(v).await,
                        Action::AppendLog(entry) => self_clone.append_log(entry),
                        Action::ClearMessages => self_clone.clear_messages(),
                        Action::ToggleShowLogs => self_clone.toggle_show_logs(),
                        Action::SetDisplayActive(active) => self_clone.set_display_active(active),
                        Action::SetAudioStatus(status) => self_clone.set_audio_status(status).await,
                        Action::SetWifiStatus(status) => self_clone.set_wifi_status(status).await,
                        Action::Shutdown => self_clone.shutdown().await,
                    };

                    let changes = {
                        let mut inner = self_clone.inner.lock().unwrap();
                        let c = inner.changes.clone();
                        inner.changes.clear();
                        c
                    };
                    let subs = self_clone.subscribers.lock().unwrap();
                    for sub in subs.iter() {
                        sub.send(changes.clone()).ok();
                    }
                }
            });
        }

        self_
    }

    pub fn set_player<P, T, E, F>(&self, player: Arc<tokio::sync::Mutex<Player<P, T, E, F>>>)
    where
        P: Fn(Progress) + Send + Sync + 'static,
        T: Fn(Option<LibraryEntry>) + Send + Sync + 'static,
        E: Fn(LibraryEntry) + Send + Sync + 'static,
        F: Fn(String) + Send + Sync + 'static,
    {
        let adapter: Box<dyn PlayerControl> = Box::new(PlayerAdapter { inner: player });
        let player_slot = self.player.clone();
        tokio::spawn(async move {
            *player_slot.lock().await = Some(adapter);
        });
    }

    pub fn dispatch(&self, action: Action) {
        let tx = self.action_tx.lock().unwrap();
        tx.send(action).ok();
    }

    pub fn subscribe<F>(&self, callback: F)
    where
        F: Fn(Changes) + Send + 'static,
    {
        let (tx, rx) = channel::<Changes>();
        {
            let mut subs = self.subscribers.lock().unwrap();
            subs.push(tx);
        }

        std::thread::spawn(move || {
            for change in rx {
                callback(change);
            }
        });
    }
}

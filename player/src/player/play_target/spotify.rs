use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use database::model::library_entry::{Model as LibraryEntry, Variant};
use librespot::core::{Session, SessionConfig, SpotifyId};
use librespot::discovery::Credentials;
use librespot::playback::audio_backend;
use librespot::playback::config::{AudioFormat, PlayerConfig};
use librespot::playback::mixer::VolumeGetter;
use librespot::playback::player::{Player, PlayerEvent};
use tokio::spawn;
use tracing::error;

use crate::player::play_target::{PlayTarget, Progress};
use crate::player::spotify_manager::SpotifyManager;

#[derive(Clone)]
struct SpotifyVolume {
    volume: Arc<Mutex<f64>>,
}
impl VolumeGetter for SpotifyVolume {
    fn attenuation_factor(&self) -> f64 {
        self.volume.lock().expect("Failed to lock volume").clone()
    }
}
impl SpotifyVolume {
    pub fn new(volume: f64) -> Self {
        Self {
            volume: Arc::new(Mutex::new(volume)),
        }
    }
    pub fn set(&mut self, volume: f64) {
        *self.volume.lock().expect("Could not lock volume") = volume;
    }
}

#[derive(Clone)]
pub struct SpotifyPlayTarget {
    volume: Box<SpotifyVolume>,
    player: Arc<Player>,
    progress: Arc<Mutex<Progress>>,
}

impl SpotifyPlayTarget {
    pub async fn new(manager: SpotifyManager, volume: f64) -> Self {
        let session_config = SessionConfig::default();
        let player_config = PlayerConfig {
            position_update_interval: Some(Duration::from_millis(250)),
            ..PlayerConfig::default()
        };
        let audio_format = AudioFormat::default();
        let session = Session::new(session_config, None);

        if let Some(token) = manager
            .client
            .token
            .lock()
            .expect("Failed to lock token")
            .as_ref()
        {
            let creds = Credentials::with_access_token(token.access_token.to_owned());
            session
                .connect(creds, true)
                .await
                .expect("Could not connect to spotify");
        };

        let backend = audio_backend::find(None).unwrap();
        let volume = Box::new(SpotifyVolume::new(volume));
        let player = Player::new(player_config, session, volume.clone(), move || {
            backend(None, audio_format)
        });

        // Receive duration and position from PlayerEvent channel
        let progress = Arc::new(Mutex::new(Progress::default()));
        {
            let progress = progress.clone();
            let player = player.clone();
            spawn(async move {
                let mut channel = player.get_player_event_channel();
                loop {
                    let event = channel.recv().await;
                    if let Some(event) = event {
                        match event {
                            PlayerEvent::Stopped { .. } => {
                                let mut progress = progress.lock().unwrap();
                                progress.duration = Duration::from_millis(0);
                                progress.position = Duration::from_millis(0);
                            }
                            PlayerEvent::Seeked { position_ms, .. } => {
                                let mut progress = progress.lock().unwrap();
                                progress.position = Duration::from_millis(position_ms as u64);
                            }
                            PlayerEvent::PositionChanged { position_ms, .. } => {
                                let mut progress = progress.lock().unwrap();
                                progress.position = Duration::from_millis(position_ms as u64);
                            }
                            PlayerEvent::TrackChanged { audio_item } => {
                                let mut progress = progress.lock().unwrap();
                                progress.duration =
                                    Duration::from_millis(audio_item.duration_ms as u64);
                                progress.position = Duration::from_millis(0);
                            }
                            PlayerEvent::EndOfTrack { .. } => {
                                let mut progress = progress.lock().unwrap();
                                progress.position = progress.duration;
                            }
                            _ => {}
                        }
                    }
                }
            });
        }

        Self {
            volume,
            player,
            progress,
        }
    }

    fn get_play_id(&self, track: &LibraryEntry) -> Result<SpotifyId, String> {
        if !matches!(track.variant, Variant::Spotify) {
            error!(
                "Attempted to play non-Spotify track on Spotify play target: {}",
                track.id
            );
            return Err("Track is not a Spotify track".to_string());
        }

        let track_source = track.track_source.as_ref().ok_or("Track source not set")?;
        let spotify_type = track_source
            .spotify_type
            .as_ref()
            .ok_or("Spotify type not set")?;
        let spotify_id = track_source
            .spotify_id
            .as_ref()
            .ok_or("Spotify ID not set")?;

        SpotifyId::from_uri(
            format!(
                "spotify:{}:{}",
                spotify_type.to_string(),
                spotify_id.to_string()
            )
            .as_str(),
        )
        .map_err(|id| format!("Invalid Spotify ID: {}", id))
    }
}

#[async_trait]
impl PlayTarget for SpotifyPlayTarget {
    async fn play(&mut self, track: &LibraryEntry) -> Result<(), String> {
        self.player.load(self.get_play_id(track)?, true, 0);
        Ok(())
    }

    async fn queue(&mut self, track: &LibraryEntry) -> Result<(), String> {
        self.player.preload(self.get_play_id(track)?);
        Ok(())
    }

    async fn pause(&mut self) -> Result<(), String> {
        self.player.pause();
        Ok(())
    }

    async fn resume(&mut self) -> Result<(), String> {
        self.player.play();
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), String> {
        self.player.stop();
        Ok(())
    }

    async fn seek_to(&mut self, position: Duration) -> Result<(), String> {
        self.player.seek(position.as_millis() as u32);
        Ok(())
    }

    async fn set_volume(&mut self, volume: f64) -> Result<(), String> {
        self.volume.set(volume);
        Ok(())
    }

    async fn get_progress(&self) -> Result<Progress, String> {
        Ok(self.progress.lock().unwrap().clone())
    }

    fn clone_box(&self) -> Box<dyn PlayTarget> {
        Box::new(self.clone())
    }
}

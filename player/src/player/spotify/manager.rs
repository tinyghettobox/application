use base64::engine::general_purpose;
use base64::Engine;
use chrono::Utc;
use librespot::core::{error::ErrorKind, Session, SessionConfig, SpotifyUri};
use librespot::discovery::Credentials;
use librespot::playback::audio_backend;
use librespot::playback::config::{AudioFormat, PlayerConfig};
use librespot::playback::player::{Player, PlayerEvent};
use reqwest::Client;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

use super::backoff::ExponentialBackoff;
use super::volume::SpotifyVolume;
use crate::player::play_target::ProgressStatus;
use crate::Progress;
use database::{
    model::spotify_config::Model as SpotifyConfig, DatabaseConnection, SpotifyConfigRepository,
};
use serde::Deserialize;

#[derive(Deserialize)]
struct RefreshResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u64,
}

#[derive(Clone)]
pub struct SpotifyManager {
    conn: DatabaseConnection,
    session: Arc<Mutex<Session>>,
    librespot: Arc<Mutex<Option<Arc<Player>>>>,
    volume: Box<SpotifyVolume>,
    progress: Arc<Mutex<Progress>>,
    loading_action: Arc<Mutex<HashMap<SpotifyUri, String>>>,
}

impl SpotifyManager {
    pub async fn new(conn: &DatabaseConnection) -> Self {
        let spotify = Self {
            conn: conn.clone(),
            session: Arc::new(Mutex::new(Session::new(SessionConfig::default(), None))),
            librespot: Default::default(),
            volume: Box::new(SpotifyVolume::new(0.0)),
            progress: Default::default(),
            loading_action: Arc::new(Mutex::new(HashMap::new())),
        };

        spotify.start_polling_config();
        spotify.start_token_refresh();
        spotify.start_observing_session();

        spotify
    }

    pub fn set_volume(&mut self, volume: f64) {
        self.volume.set(volume);
    }

    pub fn load(&self, track_id: SpotifyUri, play: bool, position_ms: u32) {
        if let Some(librespot) = self.librespot.lock().unwrap().as_ref() {
            self.loading_action
                .lock()
                .unwrap()
                .insert(track_id.clone(), "load".to_string());
            librespot.load(track_id, play, position_ms);
        }
    }

    pub fn preload(&self, track_id: SpotifyUri) {
        if let Some(librespot) = self.librespot.lock().unwrap().as_ref() {
            self.loading_action
                .lock()
                .unwrap()
                .insert(track_id.clone(), "preload".to_string());
            librespot.preload(track_id);
        }
    }

    pub fn play(&self) {
        if let Some(librespot) = self.librespot.lock().unwrap().as_ref() {
            librespot.play();
        }
    }

    pub fn pause(&self) {
        if let Some(librespot) = self.librespot.lock().unwrap().as_ref() {
            librespot.pause();
        }
    }

    pub fn stop(&self) {
        if let Some(librespot) = self.librespot.lock().unwrap().as_ref() {
            librespot.stop();
        }
    }

    pub fn seek(&self, position_ms: u32) {
        if let Some(librespot) = self.librespot.lock().unwrap().as_ref() {
            librespot.seek(position_ms);
        }
    }

    pub fn get_progress(&self) -> Progress {
        self.progress.lock().unwrap().clone()
    }

    fn start_handling_librespot_events(&self) {
        let self_ = self.clone();
        tokio::spawn(async move {
            let mut backoff = ExponentialBackoff::new(10, 2, 30);
            let librespot = self_.librespot.lock().unwrap().clone();
            if let Some(librespot) = librespot.as_ref() {
                let mut channel = librespot.get_player_event_channel();
                loop {
                    let event = channel.recv().await;
                    if let Some(event) = event {
                        debug!("Librespot event: {:?}", event);
                        match event {
                            PlayerEvent::Playing { .. } => {
                                warn!("Librespot playing event");
                                let mut progress = self_.progress.lock().unwrap();
                                progress.status = ProgressStatus::Playing;
                            }
                            PlayerEvent::Stopped { .. } => {
                                warn!("Librespot stop event");
                                let mut progress = self_.progress.lock().unwrap();
                                progress.duration = Duration::from_millis(0);
                                progress.position = Duration::from_millis(0);
                                progress.status = ProgressStatus::Stopped;
                                backoff.reset();
                            }
                            PlayerEvent::Seeked { position_ms, .. } => {
                                warn!("Librespot seeked event: {}", position_ms);
                                let mut progress = self_.progress.lock().unwrap();
                                progress.position = Duration::from_millis(position_ms as u64);
                            }
                            PlayerEvent::PositionChanged { position_ms, .. } => {
                                let mut progress = self_.progress.lock().unwrap();
                                progress.position = Duration::from_millis(position_ms as u64);
                            }
                            PlayerEvent::TrackChanged { audio_item } => {
                                warn!("Librespot track changed event: {:?}", audio_item);
                                let mut progress = self_.progress.lock().unwrap();
                                progress.duration =
                                    Duration::from_millis(audio_item.duration_ms as u64);
                                progress.position = Duration::from_millis(0);
                                progress.preloaded = false;
                                backoff.reset();
                            }
                            PlayerEvent::EndOfTrack { .. } => {
                                warn!("Librespot enf of track event");
                                let mut progress = self_.progress.lock().unwrap();
                                // let progress_percent = 100.0 / progress.duration.as_secs() as f64
                                //     * progress.position.as_secs() as f64;
                                // warn!(
                                //     "Track ended at {}%, duration: {:?}, position: {:?}",
                                //     progress_percent, progress.duration, progress.position
                                // );
                                // if progress_percent < 90 {
                                //     progress.status = ProgressStatus::Failed(
                                //         "Streamer error, track ended early".to_string(),
                                //     );
                                // } else {
                                progress.position = progress.duration;
                                // }
                            }
                            PlayerEvent::Loading { .. } => {
                                warn!("Librespot loading event");
                                // let mut progress = self_.progress.lock().unwrap();
                                // progress.status = match &progress.status {
                                //     ProgressStatus::Playing => ProgressStatus::Preloading,
                                //     _ => ProgressStatus::Loading,
                                // };
                            }
                            PlayerEvent::Preloading { .. } => {
                                warn!("Librespot preloading event");
                                let mut progress = self_.progress.lock().unwrap();
                                progress.preloaded = true;
                            }
                            PlayerEvent::Unavailable { track_id, .. } => {
                                warn!(
                                    "Librespot failed to play track, retrying {:?} attempt {}",
                                    track_id,
                                    backoff.attempts()
                                );

                                if let Some(delay) = backoff.next_delay() {
                                    sleep(delay).await;
                                    let loading_action = self_
                                        .loading_action
                                        .lock()
                                        .unwrap()
                                        .get(&track_id)
                                        .unwrap_or(&"load".to_string())
                                        .to_string();

                                    if loading_action == "preload" {
                                        self_.preload(track_id);
                                    } else {
                                        self_.load(track_id, false, 0);
                                    }
                                    continue;
                                }

                                warn!("Librespot failed to play track after 3 attempts, giving up");
                                let mut progress = self_.progress.lock().unwrap();
                                progress.duration = Duration::from_millis(0);
                                progress.position = Duration::from_millis(0);
                                progress.status =
                                    ProgressStatus::Failed("Could not load track".to_string());
                            }
                            e => {
                                debug!("Librespot event: {:?}", e);
                            }
                        }
                    }
                }
            }
        });
    }

    /// When application was started first time and no spotify credentials was configured,
    /// this function will poll for changes to the config to initialize connection to librespot.
    fn start_polling_config(&self) {
        let self_ = self.clone();
        tokio::spawn(async move {
            loop {
                {
                    let config = SpotifyConfigRepository::get(&self_.conn)
                        .await
                        .expect("Could not get spotify config");

                    if config.access_token.is_none() {
                        info!("No Spotify access token configured, skipping connection");
                        continue;
                    }

                    //
                    // debug!("Token before: {:?}", config.access_token);
                    // if let Err(error) = self_.refresh_token(&mut config).await {
                    //     warn!("Could not refresh spotify token: {}", error);
                    //     continue;
                    // }
                    // debug!("Token after: {:?}", config.access_token);

                    if self_.librespot.lock().unwrap().is_none() {
                        let player_config = PlayerConfig {
                            position_update_interval: Some(Duration::from_millis(250)),
                            ..PlayerConfig::default()
                        };
                        let audio_format = AudioFormat::default();
                        let session = match self_.spotify_connect(Some(config)).await {
                            Ok(session) => session,
                            Err(error) => {
                                panic!(
                                    "Failed to connect to Spotify after multiple attempts: {}",
                                    error
                                );
                            }
                        };
                        *self_.session.lock().unwrap() = session.clone();

                        let backend = audio_backend::find(None).unwrap();
                        let player =
                            Player::new(player_config, session, self_.volume.clone(), move || {
                                backend(None, audio_format)
                            });

                        *self_.librespot.lock().unwrap() = Some(player);

                        self_.start_handling_librespot_events();
                    }

                    break;
                }
            }
        });
    }

    fn start_observing_session(&self) {
        let self_ = self.clone();
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(1)).await;
                if self_.session.lock().unwrap().is_invalid() {
                    warn!("Spotify session disconnected, reconnecting...");
                    match self_.spotify_connect(None).await {
                        Ok(new_session) => {
                            let self_ = self_.clone();
                            *self_.session.lock().unwrap() = new_session.clone();
                            self_
                                .librespot
                                .lock()
                                .unwrap()
                                .as_mut()
                                .unwrap()
                                .set_session(new_session);
                            warn!("Spotify session reconnected");
                        }
                        Err(error) => {
                            error!("Failed to reconnect to Spotify: {}", error);
                        }
                    }
                }
            }
        });
    }

    async fn spotify_connect(&self, config: Option<SpotifyConfig>) -> Result<Session, String> {
        let mut config = match config.as_ref() {
            Some(c) => c.to_owned(),
            None => SpotifyConfigRepository::get(&self.conn)
                .await
                .expect("Could not get spotify config"),
        };

        let mut backoff = ExponentialBackoff::new(5, 2, 30);
        let session_config = SessionConfig::default();
        let session = Session::new(session_config, None);
        loop {
            let token = config.access_token.clone().unwrap();
            let creds = Credentials::with_access_token(token);
            match session.connect(creds, false).await {
                Ok(_) => {
                    debug!("Connected to Spotify successfully");
                    return Ok(session);
                }
                Err(error) => {
                    warn!("Could not connect to Spotify: {}", error);

                    if error.kind == ErrorKind::PermissionDenied
                        || error.kind == ErrorKind::Unauthenticated
                    {
                        if self.refresh_token(&mut config).await.is_ok() {
                            continue;
                        }
                    }
                    // Internal server errors might be self healing once they fix their issues
                    if error.kind == ErrorKind::Internal
                        || error.kind == ErrorKind::ResourceExhausted
                    {
                        backoff.set_max_attempts(2000);
                    }

                    match backoff.next_delay() {
                        Some(delay) => sleep(delay).await,
                        None => break,
                    }
                }
            }
        }

        Err("Failed to connect to Spotify after multiple attempts".to_string())
    }

    fn start_token_refresh(&self) {
        let self_ = self.clone();
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(360)).await;
                let mut config = SpotifyConfigRepository::get(&self_.conn)
                    .await
                    .expect("Could not get spotify config");

                if let Err(error) = self_.refresh_token(&mut config).await {
                    warn!("Could not refresh spotify token: {}", error);
                }
            }
        });
    }

    async fn refresh_token(&self, config: &mut SpotifyConfig) -> Result<(), String> {
        info!("Refreshing spotify token");

        if config.access_token.is_none() {
            info!("No spotify token defined yet");
            return Err("No spotify token defined".to_string());
        }

        // let token_expired = config
        //     .expired_at
        //     .clone()
        //     .and_then(|expired_at| DateTime::parse_from_rfc3339(&expired_at).ok())
        //     .map(|date_time| date_time < Utc::now() + Duration::from_secs(600))
        //     .ok_or("Could not parse expired date".to_string())?;
        //
        // if !token_expired {
        //     info!("Spotify token is still valid, no need to refresh");
        //     return Ok(());
        // }

        if let Some(refresh_token) = config.refresh_token.as_ref() {
            let credentials = format!("{}:{}", config.client_id, config.secret_key);
            let encoded = general_purpose::STANDARD.encode(credentials);
            debug!("Encoded credentials: {}", encoded);

            let mut params = HashMap::new();
            params.insert("grant_type", "refresh_token");
            params.insert("refresh_token", refresh_token);

            let payload = Client::new()
                .post("https://accounts.spotify.com/api/token")
                .header("Authorization", format!("Basic {}", encoded))
                .form(&params)
                .send()
                .await
                .map_err(|e| format!("Could not send request to Spotify {:?}", e))?
                .json::<RefreshResponse>()
                .await
                .map_err(|e| format!("Could not parse Spotify response: {}", e))?;

            let expires_at = Utc::now() + Duration::from_secs(payload.expires_in.to_owned());
            info!("Spotify token will expire at: {}", expires_at.to_rfc3339());

            config.access_token = Some(payload.access_token.to_owned());
            if let Some(refresh_token) = payload.refresh_token {
                config.refresh_token = Some(refresh_token.to_owned());
            }
            config.expired_at = Some(expires_at.to_rfc3339());

            SpotifyConfigRepository::update(&self.conn, config.clone())
                .await
                .expect("Could not save spotify token");

            info!("Spotify token refreshed successfully");
        } else {
            warn!("No refresh token available, cannot refresh Spotify token");
            return Err("No refresh token available".to_string());
        }
        Ok(())
    }
}

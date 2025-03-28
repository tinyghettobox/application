use std::time::Duration;

use async_trait::async_trait;
use rspotify::model::{
    AdditionalType, AlbumId, ArtistId, EpisodeId, IdError, PlayContextId, PlayableItem, PlaylistId, ShowId, TrackId,
};
use rspotify::prelude::{OAuthClient, PlayableId};
use tracing::{debug, error};

use database::model::library_entry::{Model as LibraryEntry, Variant};

use crate::player::play_target::{PlayTarget, Progress};
use crate::player::spotify_manager::SpotifyManager;

#[derive(Clone)]
pub struct SpotifyPlayTarget {
    manager: SpotifyManager,
    device_id: Option<String>,
}

impl SpotifyPlayTarget {
    pub async fn new(manager: SpotifyManager, _volume: f64) -> Self {
        Self {
            manager,
            device_id: None,
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
        let spotify_type = track_source.spotify_type.as_ref().ok_or("Spotify type not set")?;
        let spotify_id = track_source.spotify_id.as_ref().ok_or("Spotify ID not set")?;

        SpotifyId::from(spotify_type.to_string(), spotify_id.to_string())
            .map_err(|id| format!("Invalid Spotify ID: {}", id))
    }

    fn get_device_id(&mut self) -> Result<String, String> {
        if let Some(device_id) = &self.device_id {
            return Ok(device_id.to_owned());
        }

        let devices = self.manager.client.device().map_err(|e| format!("Failed to get device ID: {}", e))?;

        debug!("Found spotify devices: {:?}", devices);

        if devices.len() == 0 {
            return Err("No devices found".to_string());
        }

        self.device_id = devices
            .iter()
            .find(|device| device.is_active && device.name.contains("TinyGhettoBox"))
            .unwrap_or(&devices[0])
            .id
            .clone();

        self.device_id.clone().ok_or("Failed to get device ID".to_string())
    }
}

#[async_trait]
impl PlayTarget for SpotifyPlayTarget {
    async fn play(&mut self, track: &LibraryEntry) -> Result<(), String> {
        let device_id = self.get_device_id()?;
        let device_id = Some(device_id.as_str());
        let result = match self.get_play_id(track)? {
            SpotifyId::Playable(id) => self.manager.client.start_uris_playback(vec![id], device_id, None, None),
            SpotifyId::Context(id) => self.manager.client.start_context_playback(id, device_id, None, None),
        };

        result.map_err(|e| format!("Failed to play track: {}", e))
    }

    async fn queue(&mut self, track: &LibraryEntry) -> Result<(), String> {
        let device_id = self.get_device_id()?;
        let device_id = Some(device_id.as_str());
        let result = match self.get_play_id(track)? {
            SpotifyId::Playable(id) => self.manager.client.add_item_to_queue(id, device_id),
            SpotifyId::Context(_) => return Err("Can not queue contexts".to_string()),
        };

        result.map_err(|e| format!("Failed to play track: {}", e))
    }

    async fn pause(&mut self) -> Result<(), String> {
        let device_id = self.get_device_id()?;
        let device_id = Some(device_id.as_str());
        self.manager.client.pause_playback(device_id).map_err(|e| format!("Failed to pause playback: {}", e))
    }

    async fn resume(&mut self) -> Result<(), String> {
        let device_id = self.get_device_id()?;
        let device_id = Some(device_id.as_str());
        self.manager.client.resume_playback(device_id, None).map_err(|e| format!("Failed to resume playback: {}", e))
    }

    async fn stop(&mut self) -> Result<(), String> {
        self.pause().await
    }

    async fn seek_to(&mut self, position: Duration) -> Result<(), String> {
        let device_id = self.get_device_id()?;
        let device_id = Some(device_id.as_str());
        let duration =
            chrono::Duration::from_std(position).map_err(|e| format!("Failed to convert duration: {}", e))?;
        self.manager.client.seek_track(duration, device_id).map_err(|e| format!("Failed to seek track: {}", e))
    }

    async fn set_volume(&mut self, volume: f64) -> Result<(), String> {
        let device_id = self.get_device_id()?;
        let device_id = Some(device_id.as_str());

        self.manager
            .client
            .volume((volume * 100.0) as u8, device_id)
            .map_err(|e| format!("Failed to set volume: {}", e))
    }

    async fn get_progress(&self) -> Result<Progress, String> {
        let playback = self
            .manager
            .client
            .current_playback(None, None::<Vec<&AdditionalType>>)
            .map_err(|e| format!("Failed to get current playback position: {}", e))?
            .ok_or("No current playback returned".to_string())?;

        let progress = playback
            .progress
            .ok_or("No progress reported".to_string())?
            .to_std()
            .map_err(|e| format!("Failed to convert chrono duration: {}", e))?;

        let duration = playback
            .item
            .ok_or("No item reported".to_string())
            .map(|item| match item {
                PlayableItem::Track(track) => track.duration,
                PlayableItem::Episode(episode) => episode.duration,
            })?
            .to_std()
            .map_err(|e| format!("Failed to convert chrono duration: {}", e))?;

        Ok(Progress {
            position: progress,
            duration,
            is_finite: true,
        })
    }

    fn clone_box(&self) -> Box<dyn PlayTarget> {
        Box::new(self.clone())
    }
}

enum SpotifyId<'a> {
    Playable(PlayableId<'a>),
    Context(PlayContextId<'a>),
}

impl<'a> SpotifyId<'a> {
    fn from(spotify_type: String, spotify_id: String) -> Result<Self, IdError> {
        match spotify_type.as_str() {
            "track" => Ok(SpotifyId::Playable(PlayableId::Track(TrackId::from_id(spotify_id)?))),
            "episode" => Ok(SpotifyId::Playable(PlayableId::Episode(EpisodeId::from_id(
                spotify_id,
            )?))),
            "artist" => Ok(SpotifyId::Context(PlayContextId::Artist(ArtistId::from_id(
                spotify_id,
            )?))),
            "album" => Ok(SpotifyId::Context(PlayContextId::Album(AlbumId::from_id(spotify_id)?))),
            "playlist" => Ok(SpotifyId::Context(PlayContextId::Playlist(PlaylistId::from_id(
                spotify_id,
            )?))),
            "show" => Ok(SpotifyId::Context(PlayContextId::Show(ShowId::from_id(spotify_id)?))),
            _ => Err(IdError::InvalidType),
        }
    }
}

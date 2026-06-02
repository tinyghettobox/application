use async_trait::async_trait;
use database::model::library_entry::{Model as LibraryEntry, Variant};
use librespot::core::SpotifyUri;
use std::time::Duration;
use tracing::error;

use crate::player::play_target::{PlayTarget, Progress};
use crate::player::spotify::SpotifyManager;

#[derive(Clone)]
pub struct SpotifyPlayTarget {
    manager: SpotifyManager,
}

impl SpotifyPlayTarget {
    pub async fn new(mut manager: SpotifyManager, volume: f64) -> Self {
        manager.set_volume(volume);

        Self { manager }
    }

    fn get_spotify_uri(&self, track: &LibraryEntry) -> Result<SpotifyUri, String> {
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

        SpotifyUri::from_uri(
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
        self.manager.load(self.get_spotify_uri(track)?, true, 0);
        Ok(())
    }

    async fn queue(&mut self, track: &LibraryEntry) -> Result<(), String> {
        self.manager.preload(self.get_spotify_uri(track)?);
        Ok(())
    }

    async fn pause(&mut self) -> Result<(), String> {
        self.manager.pause();
        Ok(())
    }

    async fn resume(&mut self) -> Result<(), String> {
        self.manager.play();
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), String> {
        self.manager.stop();
        Ok(())
    }

    async fn seek_to(&mut self, position: Duration) -> Result<(), String> {
        self.manager.seek(position.as_millis() as u32);
        Ok(())
    }

    async fn set_volume(&mut self, volume: f64) -> Result<(), String> {
        self.manager.set_volume(volume);
        Ok(())
    }

    async fn get_progress(&self) -> Result<Progress, String> {
        Ok(self.manager.get_progress())
    }

    fn clone_box(&self) -> Box<dyn PlayTarget> {
        Box::new(self.clone())
    }
}

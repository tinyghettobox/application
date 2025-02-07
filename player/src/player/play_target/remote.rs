use crate::player::play_target::{PlayTarget, Progress};
use async_trait::async_trait;
use database::model::library_entry::Model as LibraryEntry;
use database::DatabaseConnection;
use kira::sound::streaming::{StreamingSoundData, StreamingSoundHandle, StreamingSoundSettings};
use kira::sound::FromFileError;
use kira::Value::Fixed;
use kira::{AudioManager, AudioManagerSettings, Decibels, DefaultBackend, Tween, Value};
use kira_remote_stream::RemoteStreamDecoder;
use std::sync::{Arc};
use tokio::sync::Mutex;
use std::time::Duration;
use tracing::debug;

#[derive(Clone)]
pub struct RemotePlayTarget {
    manager: Arc<Mutex<AudioManager<DefaultBackend>>>,
    sound_handle: Arc<Mutex<Option<StreamingSoundHandle<FromFileError>>>>,
    volume: f64,
    duration: Duration,
}

impl RemotePlayTarget {
    pub fn new(_conn: DatabaseConnection, volume: f64) -> Self {
        Self {
            manager: Arc::new(Mutex::new(
                AudioManager::<DefaultBackend>::new(AudioManagerSettings::default())
                    .expect("manager to be created"),
            )),
            sound_handle: Arc::new(Mutex::new(None)),
            volume,
            duration: Duration::default(),
        }
    }
}

fn percent_to_decibel(value: f64) -> Value<Decibels> {
    Fixed(Decibels(
        (30.0 * (value * 0.99 + 0.01).log10()) as f32,
    ))
}

#[async_trait]
impl PlayTarget for RemotePlayTarget {
    async fn play(&mut self, track: &LibraryEntry) -> Result<(), String> {
        let url = track
            .track_source
            .as_ref()
            .ok_or("Track source not set".to_string())?
            .url
            .as_ref()
            .ok_or("The url is not set on track source".to_string())?;

        debug!("Playing stream with volume: {}%/{:?}db", self.volume, percent_to_decibel(self.volume));
        let decoder = RemoteStreamDecoder::from_url(url.to_string()).await?;
        let settings = StreamingSoundSettings::default().volume(percent_to_decibel(self.volume));
        let sound = StreamingSoundData::from_decoder(decoder).with_settings(settings);
        self.duration = sound.duration();

        let handle = self
            .manager
            .lock()
            .await
            .play(sound)
            .map_err(|e| format!("Could not play sound: {}", e))?;

        *self.sound_handle.lock().await = Some(handle);

        Ok(())
    }

    async fn queue(&mut self, _track: &LibraryEntry) -> Result<(), String> {
        // Do nothing. Local playing through Kira does not support queueing, and is fast enough.
        Ok(())
    }

    async fn pause(&mut self) -> Result<(), String> {
        self.sound_handle
            .lock()
            .await
            .as_mut()
            .ok_or("No sound handle to pause".to_string())?
            .pause(Tween::default());
        Ok(())
    }

    async fn resume(&mut self) -> Result<(), String> {
        self.sound_handle
            .lock()
            .await
            .as_mut()
            .ok_or("No sound handle to resume".to_string())?
            .resume(Tween::default());
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), String> {
        self.sound_handle
            .lock()
            .await
            .as_mut()
            .ok_or("No sound handle to stop".to_string())?
            .stop(Tween::default());
        Ok(())
    }

    // TODO fix seeking for remote target
    async fn seek_to(&mut self, position: Duration) -> Result<(), String> {
        self.sound_handle
            .lock()
            .await
            .as_mut()
            .ok_or("No sound handle to pause".to_string())?
            .seek_to(position.as_secs_f64());
        Ok(())
    }

    async fn set_volume(&mut self, volume: f64) -> Result<(), String> {
        self.volume = volume;
        self.sound_handle
            .lock()
            .await
            .as_mut()
            .ok_or("No sound handle to set value".to_string())?
            .set_volume(percent_to_decibel(volume), Tween::default());
        Ok(())
    }

    async fn get_progress(&self) -> Result<Progress, String> {
        let progress = self
            .sound_handle
            .lock()
            .await
            .as_ref()
            .ok_or("No sound handle to get progress".to_string())?
            .position();

        Ok(Progress {
            position: Duration::from_secs_f64(progress),
            duration: self.duration,
        })
    }

    fn clone_box(&self) -> Box<dyn PlayTarget> {
        Box::new(self.clone())
    }
}

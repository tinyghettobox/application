use crate::player::play_target::{PlayTarget, Progress, ProgressStatus};
use async_trait::async_trait;
use database::model::library_entry::Model as LibraryEntry;
use database::DatabaseConnection;
use kira::sound::streaming::{StreamingSoundData, StreamingSoundHandle, StreamingSoundSettings};
use kira::sound::FromFileError;
use kira::Value::Fixed;
use kira::{AudioManager, AudioManagerSettings, Decibels, DefaultBackend, Tween, Value};
use kira_remote_stream::RemoteStreamDecoder;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
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
    let db = Decibels::SILENCE.0 + Decibels::SILENCE.0.abs() * value.powf(2.0) as f32;
    debug!("Setting decibels to {}", db);
    Fixed(Decibels(db))
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

        debug!(
            "Playing stream with volume: {}%/{:?}db",
            self.volume,
            percent_to_decibel(self.volume)
        );
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

        let mut handle = self.sound_handle.lock().await;

        if let Some(handle) = handle.as_mut() {
            handle.set_volume(percent_to_decibel(volume), Tween::default());
        }
        Ok(())
    }

    async fn get_progress(&self) -> Result<Progress, String> {
        if let Some(handle) = self.sound_handle.lock().await.as_mut() {
            let error = handle.pop_error();
            let progress = handle.position();
            let is_finite = self.duration.as_secs() < i32::MAX as u64;

            Ok(Progress {
                position: Duration::from_secs_f64(progress),
                duration: if is_finite {
                    self.duration
                } else {
                    Duration::from_secs_f64(progress)
                },
                is_finite, // infinite stream will have u64::MAX / sample rate as duration
                preloaded: false,
                status: if let Some(err) = error {
                    debug!("Error in stream: {}", err);
                    ProgressStatus::Failed(format!("{}", err))
                } else if handle.state().is_advancing() {
                    ProgressStatus::Playing
                } else {
                    ProgressStatus::Stopped
                },
            })
        } else {
            Ok(Progress {
                position: Duration::from_secs(0),
                duration: Duration::from_secs(0),
                is_finite: true,
                preloaded: false,
                status: ProgressStatus::Stopped,
            })
        }
    }

    fn clone_box(&self) -> Box<dyn PlayTarget> {
        Box::new(self.clone())
    }
}

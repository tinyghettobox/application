use std::sync::{Arc, Mutex};
use database::model::library_entry::Model as LibraryEntry;
use std::time::Duration;
use async_trait::async_trait;
use kira::manager::{AudioManager, AudioManagerSettings};
use kira::manager::backend::DefaultBackend;
use kira::sound::FromFileError;
use kira::sound::streaming::{StreamingSoundData, StreamingSoundHandle, StreamingSoundSettings};
use kira::tween::Tween;
use kira::tween::Value::Fixed;
use kira::Volume;
use database::DatabaseConnection;
use kira_remote_stream::RemoteStreamDecoder;
use crate::player::play_target::{PlayTarget, Progress};

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
            manager: Arc::new(Mutex::new(AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).expect("manager to be created"))),
            sound_handle: Arc::new(Mutex::new(None)),
            volume,
            duration: Duration::default(),
        }
    }
}

#[async_trait]
impl PlayTarget for RemotePlayTarget {
    async fn play(&mut self, track: &LibraryEntry) -> Result<(), String> {
        let url = track.track_source
            .as_ref()
            .ok_or("Track source not set".to_string())?
            .url
            .as_ref()
            .ok_or("The url is not set on track source".to_string())?;

        let decoder = RemoteStreamDecoder::from_url(url.to_string()).await?;
        let settings = StreamingSoundSettings::default().volume(Fixed(Volume::Amplitude(self.volume)));
        let sound = StreamingSoundData::from_decoder(decoder, settings);
        self.duration = sound.duration();

        let handle = self.manager
            .lock()
            .map_err(|e| format!("Could not lock audio manager: {}", e))?
            .play(sound)
            .map_err(|e| format!("Could not play sound: {}", e))?;
        self.sound_handle = Arc::new(Mutex::new(Some(handle)));

        Ok(())
    }

    async fn queue(&mut self, _track: &LibraryEntry) -> Result<(), String> {
        // Do nothing. Local playing through Kira does not support queueing, and is fast enough.
        Ok(())
    }

    async fn pause(&mut self) -> Result<(), String> {
        self.sound_handle
            .lock()
            .map_err(|e| format!("Could not lock sound handle: {}", e))?
            .as_mut()
            .ok_or("No sound handle to pause".to_string())?
            .pause(Tween::default())
            .map_err(|e| format!("Could not pause sound: {}", e))
    }

    async fn resume(&mut self) -> Result<(), String> {
        self.sound_handle
            .lock()
            .map_err(|e| format!("Could not lock sound handle: {}", e))?
            .as_mut()
            .ok_or("No sound handle to resume".to_string())?
            .resume(Tween::default())
            .map_err(|e| format!("Could not resume sound: {}", e))
    }

    async fn stop(&mut self) -> Result<(), String> {
        self.sound_handle
            .lock()
            .map_err(|e| format!("Could not lock sound handle: {}", e))?
            .as_mut()
            .ok_or("No sound handle to stop".to_string())?
            .stop(Tween::default())
            .map_err(|e| format!("Could not stop sound: {}", e))
    }

    async fn seek_to(&mut self, position: Duration) -> Result<(), String> {
        self.sound_handle
            .lock()
            .map_err(|e| format!("Could not lock sound handle: {}", e))?
            .as_mut()
            .ok_or("No sound handle to pause".to_string())?
            .seek_to(position.as_secs_f64())
            .map_err(|e| format!("Could not seek to position: {}", e))
    }

    async fn set_volume(&mut self, volume: f64) -> Result<(), String> {
        self.sound_handle
            .lock()
            .map_err(|e| format!("Could not lock sound handle: {}", e))?
            .as_mut()
            .ok_or("No sound handle to set value".to_string())?
            .set_volume(Volume::Amplitude(volume), Tween::default())
            .map_err(|e| format!("Could not set value: {}", e))
    }

    async fn get_progress(&self) -> Result<Progress, String> {
        let progress = self.sound_handle
            .lock()
            .map_err(|e| format!("Could not lock sound handle: {}", e))?
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

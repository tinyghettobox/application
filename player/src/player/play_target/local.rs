use crate::player::play_target::{PlayTarget, Progress};
use async_trait::async_trait;
use database::model::library_entry::Model as LibraryEntry;
use database::{DatabaseConnection, TrackSourceRepository};
use kira::manager::backend::DefaultBackend;
use kira::manager::{AudioManager, AudioManagerSettings};
use kira::sound::streaming::{StreamingSoundData, StreamingSoundHandle, StreamingSoundSettings};
use kira::sound::FromFileError;
use kira::tween::Tween;
use kira::tween::Value::Fixed;
use kira::Volume;
use std::io::{Read, Seek, SeekFrom};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use symphonia::core::io::MediaSource;

#[derive(Clone)]
pub struct LocalPlayTarget {
    conn: DatabaseConnection,
    manager: Arc<Mutex<AudioManager<DefaultBackend>>>,
    sound_handle: Arc<Mutex<Option<StreamingSoundHandle<FromFileError>>>>,
    volume: f64,
    duration: Duration,
}

impl LocalPlayTarget {
    pub async fn new(conn: DatabaseConnection, volume: f64) -> Self {
        Self {
            conn,
            manager: Arc::new(Mutex::new(
                AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()).expect("manager to be created"),
            )),
            sound_handle: Arc::new(Mutex::new(None)),
            volume,
            duration: Duration::default(),
        }
    }
}

#[async_trait]
impl PlayTarget for LocalPlayTarget {
    async fn play(&mut self, track: &LibraryEntry) -> Result<(), String> {
        let file = TrackSourceRepository::get_file(&self.conn, track.track_source.as_ref().unwrap().id)
            .await
            .map_err(|e| format!("Could not get file: {}", e))?
            .ok_or("Track source has no file set".to_string())?;

        let media_source = BytesMediaSource::new(file.clone());
        let settings = StreamingSoundSettings::default().volume(Fixed(Volume::Amplitude(self.volume)));
        let sound = StreamingSoundData::from_media_source(media_source, settings)
            .map_err(|e| format!("Could not create sound data: {}", e))?;
        self.duration = sound.duration();

        let handle = self
            .manager
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
        let progress = self
            .sound_handle
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

struct BytesMediaSource {
    bytes: Vec<u8>,
    position: u64,
}

impl BytesMediaSource {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes, position: 0 }
    }
}

impl Read for BytesMediaSource {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let bytes = &self.bytes[self.position as usize..];
        let bytes_to_read = std::cmp::min(buf.len(), bytes.len());

        buf[..bytes_to_read].copy_from_slice(&bytes[..bytes_to_read]);

        self.position += bytes_to_read as u64;

        Ok(bytes_to_read)
    }
}

impl Seek for BytesMediaSource {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let new_position = match pos {
            SeekFrom::Start(offset) => offset as i64,
            SeekFrom::End(offset) => self.bytes.len() as i64 + offset,
            SeekFrom::Current(offset) => self.position as i64 + offset,
        };

        if new_position < 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Seek position to low",
            ));
        }
        if new_position as usize > self.bytes.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Seek position to high",
            ));
        }

        self.position = new_position as u64;

        Ok(self.position)
    }
}

impl MediaSource for BytesMediaSource {
    fn is_seekable(&self) -> bool {
        true
    }

    fn byte_len(&self) -> Option<u64> {
        Some(self.bytes.len() as u64)
    }
}

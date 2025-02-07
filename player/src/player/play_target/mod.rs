use database::model::library_entry::Model as LibraryEntry;
use std::time::Duration;
use async_trait::async_trait;

mod spotify;
mod local;
mod remote;

pub use spotify::SpotifyPlayTarget;
pub use local::LocalPlayTarget;
pub use remote::RemotePlayTarget;

#[derive(Clone, Debug)]
pub struct Progress {
    pub position: Duration,
    pub duration: Duration
}

impl Default for Progress {
    fn default() -> Self {
        Self {
            position: Duration::from_secs(0),
            duration: Duration::from_secs(1)
        }
    }
}

impl Progress {
    pub fn as_f64(&self) -> f64 {
        // For infinite streams the duration will be u64::MAX / sample rate
        // Infinite streams should be always at 100% progress allowing to seek back
        if self.duration > Duration::from_secs(i32::MAX as u64) {
            return 100.0
        }

        self.position.as_secs_f64() / self.duration.as_secs_f64() * 100.0
    }
}

#[async_trait]
pub trait PlayTarget {
    async fn play(&mut self, track: &LibraryEntry) -> Result<(), String>;
    async fn queue(&mut self, track: &LibraryEntry) -> Result<(), String>;
    async fn pause(&mut self) -> Result<(), String>;
    async fn resume(&mut self) -> Result<(), String>;
    async fn stop(&mut self) -> Result<(), String>;
    async fn seek_to(&mut self, position: Duration) -> Result<(), String>;
    async fn set_volume(&mut self, volume: f64) -> Result<(), String>;
    async fn get_progress(&self) -> Result<Progress, String>;
    fn clone_box(&self) -> Box<dyn PlayTarget>;
}

impl Clone for Box<dyn PlayTarget> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}


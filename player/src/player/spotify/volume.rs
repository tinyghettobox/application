use librespot::playback::mixer::VolumeGetter;
use std::sync::{Arc, Mutex};

/// Struct us used to pass volume into librespot
#[derive(Clone)]
pub(crate) struct SpotifyVolume {
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

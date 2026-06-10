use database::model::library_entry::{Model as LibraryEntry, Variant};

use crate::model::{actions::{Action, AudioStatus}, state::State, state::Field};

impl State {
    pub(in crate::model) fn start_playing_library_entry(&mut self, library_entry: LibraryEntry) {
        // Block playback until the audio system is confirmed ready.
        if self.inner.lock().unwrap().audio_status != AudioStatus::Ready {
            let mut inner = self.inner.lock().unwrap();
            let mut msgs = inner.messages.clone();
            msgs.push("Audio system is not ready yet. Please wait.".to_string());
            inner.set(Field::messages(msgs));
            return;
        }

        // Block Spotify / HTTP-stream playback when the network is unavailable.
        if matches!(library_entry.variant, Variant::Spotify | Variant::Stream) {
            if !self.inner.lock().unwrap().wifi_status.is_network_available() {
                let mut inner = self.inner.lock().unwrap();
                let mut msgs = inner.messages.clone();
                msgs.push("Network not available. Cannot play this item.".to_string());
                inner.set(Field::messages(msgs));
                return;
            }
        }

        self.inner.lock().unwrap().set(Field::is_loading(true));
        self.dispatch(Action::PlayLibraryEntry(library_entry));
    }
}

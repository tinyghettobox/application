use database::{model::library_entry::Model as LibraryEntry, LibraryEntryRepository};
use player::Progress;
use tracing::info;

use crate::model::state::{Field, State};

impl State {
    pub(in crate::model) async fn set_playing_track(&mut self, entry: Option<LibraryEntry>) {
        match &entry {
            Some(e) => info!("Now playing: {} (id={})", e.name, e.id),
            None => info!("Playback stopped"),
        }
        let playing = entry.is_some();
        let ancestor_ids = match entry.as_ref().map(|e| e.id) {
            Some(id) => LibraryEntryRepository::get_ancestor_ids(&self.conn, id)
                .await
                .unwrap_or_default(),
            None => Vec::new(),
        };
        let mut inner = self.inner.lock().unwrap();
        inner.set(Field::playing_library_entry(entry));
        inner.set(Field::playing_ancestor_ids(ancestor_ids));
        inner.set(Field::is_playing(playing));
        inner.set(Field::is_loading(false));
    }

    pub(in crate::model) fn set_progress(&mut self, progress: Progress) {
        self.inner.lock().unwrap().set(Field::progress(progress));
    }
}

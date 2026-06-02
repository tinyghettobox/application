use database::model::library_entry::Model as LibraryEntry;

use crate::model::{actions::Action, state::State, state::Field};

impl State {
    pub(in crate::model) fn start_playing_library_entry(&mut self, library_entry: LibraryEntry) {
        self.inner.lock().unwrap().set(Field::is_loading(true));
        self.dispatch(Action::PlayLibraryEntry(library_entry));
    }
}
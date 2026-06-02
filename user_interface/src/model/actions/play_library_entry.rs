use database::{
    model::library_entry::{Model as LibraryEntry, Variant},
    LibraryEntryRepository,
};
use player::Queue;
use tracing::{error, info};

use crate::model::state::{Field, State};

impl State {
    pub(in crate::model) async fn play_library_entry(&mut self, library_entry: LibraryEntry) {
        info!("Playing library entry: {} (id={})", library_entry.name, library_entry.id);
        let parent_id = match library_entry.variant {
            Variant::Folder => library_entry.id,
            _ => library_entry.parent_id.unwrap_or(library_entry.id),
        };

        match LibraryEntryRepository::get_tracks_in_parent(&self.conn, parent_id).await {
            Ok(entries) => {
                let start_idx = entries
                    .iter()
                    .position(|e| e.id == library_entry.id)
                    .unwrap_or(0);
                let mut queue = Queue::from_iter(entries);
                queue.set_current(start_idx as i32);
                let result = match self.player.lock().await.as_mut() {
                    Some(p) => p.play_queue(queue).await,
                    None => Err("Player not yet initialized".to_string()),
                };
                if let Err(err) = result {
                    error!("Could not play track: {}", err);
                    let mut inner = self.inner.lock().unwrap();
                    inner.set(Field::is_loading(false));
                    let mut msgs = inner.messages.clone();
                    msgs.push(format!("Could not play track: {}", err));
                    inner.set(Field::messages(msgs));
                }
            }
            Err(e) => {
                error!("Could not load tracks for entry {}: {}", library_entry.id, e);
                let mut inner = self.inner.lock().unwrap();
                inner.set(Field::is_loading(false));
                let mut msgs = inner.messages.clone();
                msgs.push(format!("Could not load tracks: {}", e));
                inner.set(Field::messages(msgs));
            }
        }
    }
}
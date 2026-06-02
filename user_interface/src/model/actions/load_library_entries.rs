use database::LibraryEntryRepository;
use tracing::{info, warn};

use crate::model::state::{Field, State};

impl State {
    pub(in crate::model) async fn load_library_entry(&mut self, id: i32) {
        info!("Loading library entry {}", id);
        match LibraryEntryRepository::get(&self.conn, id).await {
            Ok(Some(mut entry)) => {
                if let Some(children) = entry.children.as_mut() {
                    let folder_ids: Vec<i32> = children
                        .iter()
                        .filter(|e| e.variant == database::model::library_entry::Variant::Folder)
                        .map(|e| e.id)
                        .collect();

                    if !folder_ids.is_empty() {
                        if let Ok(progress_map) = LibraryEntryRepository::get_folder_play_progress(
                            &self.conn,
                            folder_ids,
                        )
                        .await
                        {
                            for child in children.iter_mut() {
                                if child.variant == database::model::library_entry::Variant::Folder {
                                    if let Some(&(played, total)) = progress_map.get(&child.id) {
                                        child.play_progress = Some(
                                            if total == 0 { 0 } else { played * 100 / total },
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
                self.inner.lock().unwrap().set(Field::active_library_entry(Some(entry)));
            }
            Ok(None) => {
                warn!("Library entry {} not found", id);
                let mut inner = self.inner.lock().unwrap();
                let mut msgs = inner.messages.clone();
                msgs.push(format!("Library entry with id {} not found", id));
                inner.set(Field::messages(msgs));
            }
            Err(e) => {
                tracing::error!("Error loading library entry {}: {}", id, e);
                let mut inner = self.inner.lock().unwrap();
                let mut msgs = inner.messages.clone();
                msgs.push(format!("Error loading library entry {}: {}", id, e));
                inner.set(Field::messages(msgs));
            }
        }
    }
}

use crate::model::state::State;

impl State {
    pub(in crate::model) async fn set_played_at(&mut self, id: i32, played: bool) {
        let played_at = if played { Some(chrono::Utc::now()) } else { None };
        database::LibraryEntryRepository::mark_played(&self.conn, vec![id], played_at)
            .await
            .ok();

        if let Some(active) = self.active_library_entry() {
            self.load_library_entry(active.id).await;
        }
    }
}

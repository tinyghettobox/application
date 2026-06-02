use super::LogEntry;
use crate::model::state::{Field, State, LOG_RING_CAPACITY};

impl State {
    pub(in crate::model) fn toggle_show_logs(&mut self) {
        let current = self.inner.lock().unwrap().show_logs;
        self.inner.lock().unwrap().set(Field::show_logs(!current));
    }
}

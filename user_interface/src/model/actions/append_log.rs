use super::LogEntry;
use crate::model::state::{Field, State, LOG_RING_CAPACITY};

impl State {
    pub(in crate::model) fn append_log(&mut self, entry: LogEntry) {
        let mut inner = self.inner.lock().unwrap();
        let mut entries = inner.log_entries.clone();
        entries.push(entry);
        if entries.len() > LOG_RING_CAPACITY {
            entries.remove(0);
        }
        inner.set(Field::log_entries(entries));
    }
}

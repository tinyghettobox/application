use crate::model::state::{Field, State};

impl State {
    pub(in crate::model) fn clear_messages(&mut self) {
        self.inner.lock().unwrap().set(Field::messages(Vec::new()));
    }
}

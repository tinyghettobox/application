use crate::model::{
    actions::WifiStatus,
    state::{Field, State},
};

impl State {
    pub(in crate::model) async fn set_wifi_status(&mut self, status: WifiStatus) {
        self.inner.lock().unwrap().set(Field::wifi_status(status));
    }
}

use database::SystemConfigRepository;

use crate::model::state::{Field, State};

impl State {
    pub(in crate::model) async fn load_system_config(&mut self) {
        match SystemConfigRepository::get(&self.conn).await {
            Ok(config) => {
                self.inner
                    .lock()
                    .unwrap()
                    .set(Field::system_config(config));
            }
            Err(e) => {
                tracing::warn!("Could not load system_config: {}", e);
            }
        }
    }
}

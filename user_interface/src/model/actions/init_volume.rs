use database::SystemConfigRepository;

use crate::model::state::{Field, State};

impl State {
    pub(in crate::model) async fn init_volume(&mut self) {
        match SystemConfigRepository::get_volume(&self.conn).await {
            Ok(volume) => {
                let volume = volume as i32;
                if let Some(p) = self.player.lock().await.as_mut() {
                    p.set_volume(volume as f64 / 100.0).await.ok();
                }
                self.inner.lock().unwrap().set(Field::volume(volume));
            }
            Err(e) => {
                let mut inner = self.inner.lock().unwrap();
                let mut msgs = inner.messages.clone();
                msgs.push(format!("Could not load volume: {}", e));
                inner.set(Field::messages(msgs));
            }
        }
    }
}

use database::SystemConfigRepository;

use crate::model::state::{Field, State};

impl State {
    pub(in crate::model) async fn set_volume(&mut self, volume: i32) {
        let result = match self.player.lock().await.as_mut() {
            Some(p) => p.set_volume(volume as f64 / 100.0).await,
            None => Err("Player not yet initialized".to_string()),
        };
        match result {
            Ok(_) => {
                self.inner.lock().unwrap().set(Field::volume(volume));
                SystemConfigRepository::set_volume(&self.conn, volume as u8).await.ok();
            }
            Err(error) => {
                let mut inner = self.inner.lock().unwrap();
                let mut msgs = inner.messages.clone();
                msgs.push(format!("Could not set volume: {}", error));
                inner.set(Field::messages(msgs));
            }
        }
    }
}

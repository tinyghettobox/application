use tracing::{info, warn};

use crate::model::state::{Field, State};

impl State {
    pub(in crate::model) async fn toggle_play(&mut self, should_play: bool) {
        let result = match self.player.lock().await.as_mut() {
            Some(p) => {
                if should_play {
                    p.resume().await
                } else {
                    p.pause().await
                }
            }
            None => Err("Player not yet initialized".to_string()),
        };

        let mut inner = self.inner.lock().unwrap();
        match result {
            Ok(_) => {
                info!("{}", if should_play { "Resumed" } else { "Paused" });
                inner.set(Field::is_playing(should_play));
            }
            Err(error) => {
                warn!("Could not toggle play: {}", error);
                let mut msgs = inner.messages.clone();
                msgs.push(format!("Could not toggle play: {}", error));
                inner.set(Field::messages(msgs));
            }
        }
    }
}
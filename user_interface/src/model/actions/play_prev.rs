use crate::model::state::{Field, State};

impl State {
    pub(in crate::model) async fn play_prev(&mut self) {
        let result = match self.player.lock().await.as_mut() {
            Some(p) => p.play_prev().await,
            None => Err("Player not yet initialized".to_string()),
        };
        match result {
            Ok(Some(_)) => {
                let mut inner = self.inner.lock().unwrap();
                inner.set(Field::is_playing(true));
            }
            Ok(None) => {}
            Err(error) => {
                let mut inner = self.inner.lock().unwrap();
                let mut msgs = inner.messages.clone();
                msgs.push(format!("Could not play previous track: {}", error));
                inner.set(Field::messages(msgs));
            }
        }
    }
}

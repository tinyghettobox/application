use crate::model::state::{Field, State};

impl State {
    pub(in crate::model) async fn play_next(&mut self) {
        let result = match self.player.lock().await.as_mut() {
            Some(p) => p.play_next().await,
            None => Err("Player not yet initialized".to_string()),
        };
        match result {
            Ok(Some(_)) => {
                let mut inner = self.inner.lock().unwrap();
                inner.set(Field::is_playing(true));
            }
            Ok(None) => {
                let mut inner = self.inner.lock().unwrap();
                inner.set(Field::is_playing(false));
                inner.set(Field::playing_library_entry(None));
            }
            Err(error) => {
                let mut inner = self.inner.lock().unwrap();
                let mut msgs = inner.messages.clone();
                msgs.push(format!("Could not play next track: {}", error));
                inner.set(Field::messages(msgs));
            }
        }
    }
}

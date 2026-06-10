use crate::model::{
    actions::AudioStatus,
    state::{Field, State},
};

impl State {
    pub(in crate::model) async fn set_audio_status(&mut self, status: AudioStatus) {
        // The status icon in the navbar communicates the audio state to the user;
        // no persistent message is pushed here.
        self.inner.lock().unwrap().set(Field::audio_status(status));
    }
}

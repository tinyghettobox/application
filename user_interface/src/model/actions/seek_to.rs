use crate::model::state::State;

impl State {
    pub(in crate::model) async fn seek_to(&mut self, pct: f64) {
        let duration = self.progress().duration;
        let secs = duration.as_secs_f64() * pct / 100.0;
        if let Some(p) = self.player.lock().await.as_mut() {
            p.seek_to(secs).await.ok();
        }
    }
}

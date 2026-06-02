use crate::model::state::{Field, State};

impl State {
    pub(in crate::model) async fn shutdown(&mut self) {
        if cfg!(target_os = "linux") {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            let _ = std::process::Command::new("shutdown").arg("now").spawn();
        }
    }
}

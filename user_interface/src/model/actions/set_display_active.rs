use crate::model::state::{Field, State};

impl State {
    pub(in crate::model) fn set_display_active(&mut self, active: bool) {
        if cfg!(target_os = "linux") {
            std::fs::read_dir("/sys/class/backlight/")
                .into_iter()
                .flatten()
                .filter_map(|e| e.ok())
                .map(|e| e.path().join("bl_power"))
                .filter(|p| p.exists())
                .for_each(|p| {
                    let _ = std::fs::write(p, if active { "0" } else { "1" });
                });
        }
        self.inner.lock().unwrap().set(Field::display_active(active));
    }
}

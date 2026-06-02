use slint::{ComponentHandle, ModelRc, SharedString, VecModel, Weak};

use crate::model::actions::{Action, LogLevel};
use crate::model::{Field, State};
use crate::view_model::update_ui;
use crate::{AppWindow, Logs, UILogEntry};

pub struct LogsVM {
    ui: Weak<AppWindow>,
    state: State,
}

impl LogsVM {
    pub fn new(ui: Weak<AppWindow>, state: State) -> Self {
        let vm = LogsVM { ui, state };
        vm.setup_ui();
        vm.setup_state_listeners();
        vm
    }

    fn setup_ui(&self) {
        if let Some(ui) = self.ui.upgrade() {
            let logs = ui.global::<Logs>();
            logs.set_entries(ModelRc::new(VecModel::default()));
            logs.set_filter_level("info".into());
            logs.set_visible(false);

            {
                let state = self.state.clone();
                logs.on_close(move || {
                    state.dispatch(Action::ToggleShowLogs);
                });
            }
            {
                let ui_weak = self.ui.clone();
                logs.on_set_filter(move |level| {
                    update_ui(&ui_weak, move |ui| {
                        ui.global::<Logs>().set_filter_level(level);
                    });
                });
            }
            {
                let state = self.state.clone();
                ui.global::<crate::Navbar>().on_open_logs(move || {
                    state.dispatch(Action::ToggleShowLogs);
                });
            }
        }
    }

    fn setup_state_listeners(&self) {
        let ui_weak = self.ui.clone();
        let state_clone = self.state.clone();

        self.state.subscribe(move |changes| {
            let has_logs = changes.iter().any(|f| matches!(f, Field::log_entries(_)));
            let has_show = changes.iter().any(|f| matches!(f, Field::show_logs(_)));

            if !has_logs && !has_show {
                return;
            }

            let entries = state_clone.log_entries();
            let show = state_clone.show_logs();

            update_ui(&ui_weak, move |ui| {
                let logs = ui.global::<Logs>();
                if has_show {
                    logs.set_visible(show);
                }
                if has_logs {
                    let ui_entries: Vec<UILogEntry> = entries
                        .iter()
                        .map(|e| UILogEntry {
                            level: SharedString::from(match e.level {
                                LogLevel::Error => "error",
                                LogLevel::Warn => "warn",
                                LogLevel::Info => "info",
                                LogLevel::Debug => "debug",
                            }),
                            message: e.message.clone().into(),
                            timestamp: e.timestamp.format("%H:%M:%S").to_string().into(),
                        })
                        .collect();
                    logs.set_entries(ModelRc::new(VecModel::from(ui_entries)));
                }
            });
        });
    }
}

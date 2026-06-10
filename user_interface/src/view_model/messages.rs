use slint::{ComponentHandle, Weak};
use crate::{AppWindow, Messages};
use crate::model::{Field, State, actions::Action};
use crate::view_model::update_ui;

/// How long a toast notification stays visible before it is cleared.
const DISMISS_AFTER: std::time::Duration = std::time::Duration::from_secs(3);

pub struct MessagesVM {
    ui: Weak<AppWindow>,
    state: State,
}

impl MessagesVM {
    pub fn new(ui: Weak<AppWindow>, state: State) -> Self {
        let vm = MessagesVM { ui, state };
        vm.setup_ui();
        vm.setup_state_listeners();
        vm
    }

    pub fn setup_ui(&self) {
        if let Some(ui) = self.ui.upgrade() {
            let messages_global = ui.global::<Messages>();
            messages_global.set_messages(slint::ModelRc::new(slint::VecModel::default()));
        }
    }

    pub fn setup_state_listeners(&self) {
        let ui_weak = self.ui.clone();
        let state_clone = self.state.clone();

        self.state.subscribe(move |changes| {
            if !changes.iter().any(|f| matches!(f, Field::messages(_))) {
                return;
            }

            let messages = state_clone.messages();
            let is_non_empty = !messages.is_empty();

            // Push to the UI immediately.
            let messages_data: Vec<slint::SharedString> =
                messages.iter().map(|msg| msg.clone().into()).collect();
            update_ui(&ui_weak, move |ui| {
                ui.global::<Messages>()
                    .set_messages(slint::ModelRc::new(slint::VecModel::from(messages_data)));
            });

            // Schedule auto-dismiss. The subscriber runs on a plain std::thread
            // (no Tokio reactor), so we use std::thread::sleep rather than
            // tokio::spawn to avoid a "no reactor running" panic.
            if is_non_empty {
                let state_for_dismiss = state_clone.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(DISMISS_AFTER);
                    state_for_dismiss.dispatch(Action::ClearMessages);
                });
            }
        });
    }
}

impl Clone for MessagesVM {
    fn clone(&self) -> Self {
        MessagesVM {
            ui: self.ui.clone(),
            state: self.state.clone(),
        }
    }
}
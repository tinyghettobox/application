use slint::{ComponentHandle, Weak};
use crate::{AppWindow, Messages};
use crate::model::{Field, State};
use crate::view_model::update_ui;

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
        // Setup UI logic here
        if let Some(ui) = self.ui.upgrade() {
            let messages_global = ui.global::<Messages>();
            // Initialize with empty data
            messages_global.set_messages(slint::ModelRc::new(slint::VecModel::default()));
        }
    }

    pub fn setup_state_listeners(&self) {
        // Clone for move closure
        let ui_weak = self.ui.clone();
        let state_clone = self.state.clone();
        
        self.state.subscribe(move |changes| {
            if !changes.iter().any(|f| matches!(f, Field::messages(_))) {
                return;
            }

            let messages = state_clone.messages();
            update_ui(&ui_weak, move |ui| {
                let messages_data: Vec<slint::SharedString> =
                    messages.iter().map(|msg| msg.clone().into()).collect();
                ui.global::<Messages>()
                    .set_messages(slint::ModelRc::new(slint::VecModel::from(messages_data)));
            });
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
use slint::{ComponentHandle, Weak};

use crate::{model::{Field, State}, view_model::update_ui, AppWindow, Navbar};


pub struct NavbarVM {
    ui: Weak<AppWindow>,
    state: State,
}


impl NavbarVM {
    pub fn new(ui: Weak<AppWindow>, state: State) -> Self {
        let vm = NavbarVM { ui, state };
        vm.setup_ui();
        vm.setup_state_listeners();
        vm
    }

    pub fn setup_ui(&self) {
        // Setup UI logic here
        if let Some(ui) = self.ui.upgrade() {
            let navbar_global = ui.global::<Navbar>();
            navbar_global.set_visible(false);

            {
                let state_ = self.state.clone();
                navbar_global.on_go_back(move |parent_id| {
                    state_.dispatch(crate::model::actions::Action::LoadLibraryEntry(parent_id));
                });
            }
        }
    }

    pub fn setup_state_listeners(&self) {
        // Clone for move closure
        let ui_weak = self.ui.clone();
        let state_clone = self.state.clone();
        
        self.state.subscribe(move |changes| {
            if !changes.iter().any(|f| matches!(f, Field::active_library_entry(_))) {
                return;
            }

            let entry = match state_clone.active_library_entry() {
                Some(e) => e,
                None => return,
            };

            update_ui(&ui_weak, move |ui| {
                let navbar = ui.global::<Navbar>();
                navbar.set_visible(entry.id != 0);
                navbar.set_entry_name(entry.name.into());
                navbar.set_parent_id(entry.parent_id.unwrap_or(0));
            });
        });
    }
}
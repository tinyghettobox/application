use slint::{ComponentHandle, Weak};

use crate::{
    model::{
        actions::{AudioStatus, WifiStatus},
        Field, State,
    },
    view_model::update_ui,
    AppWindow, Navbar,
};


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
        if let Some(ui) = self.ui.upgrade() {
            let navbar_global = ui.global::<Navbar>();
            navbar_global.set_visible(false);

            {
                let state_ = self.state.clone();
                navbar_global.on_go_back(move |parent_id| {
                    state_.dispatch(crate::model::actions::Action::LoadLibraryEntry(parent_id));
                });
            }

            {
                let state_ = self.state.clone();
                navbar_global.on_open_logs(move || {
                    state_.dispatch(crate::model::actions::Action::ToggleShowLogs);
                });
            }
        }
    }

    pub fn setup_state_listeners(&self) {
        // ── Library entry changed → update title / visibility ────────────────
        {
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

        // ── Audio / wifi status changed → update status icon ─────────────────
        {
            let ui_weak = self.ui.clone();
            let state_clone = self.state.clone();

            self.state.subscribe(move |changes| {
                let relevant = changes.iter().any(|f| {
                    matches!(f, Field::audio_status(_) | Field::wifi_status(_))
                });
                if !relevant {
                    return;
                }

                let audio = state_clone.audio_status();
                let wifi = state_clone.wifi_status();
                let (icon, tooltip) = system_status_icon(&audio, &wifi);

                update_ui(&ui_weak, move |ui| {
                    let navbar = ui.global::<Navbar>();
                    navbar.set_system_status(icon.into());
                    navbar.set_status_tooltip_text(tooltip.into());
                });
            });
        }
    }
}

/// Derive the Slint icon key and human-readable tooltip from the current
/// audio + wifi state combination.
fn system_status_icon(audio: &AudioStatus, wifi: &WifiStatus) -> (&'static str, String) {
    match (audio, wifi) {
        // Both still initialising
        (AudioStatus::Initializing, WifiStatus::Initializing) => (
            "system-not-ready",
            "Initialising audio and network…".to_string(),
        ),

        // Audio failed (wifi may or may not be ready)
        (AudioStatus::Failed, _) => (
            "audio-not-ready",
            "Audio system unavailable. Music playback is disabled.".to_string(),
        ),

        // Audio still initialising but wifi is already known
        (AudioStatus::Initializing, _) => (
            "audio-not-ready",
            "Audio system is starting up…".to_string(),
        ),

        // Audio ready — now reflect wifi state
        (AudioStatus::Ready, WifiStatus::Initializing) => (
            "wifi-not-ready",
            "Checking network connectivity…".to_string(),
        ),
        (AudioStatus::Ready, WifiStatus::Disconnected) => (
            "wifi-disconnected",
            "No network connection. Spotify and streams are unavailable.".to_string(),
        ),
        (AudioStatus::Ready, WifiStatus::Weak(rssi)) => (
            "wifi-weak",
            format!("Weak signal ({} dBm). Playback may be unstable.", rssi),
        ),
        (AudioStatus::Ready, WifiStatus::Good(rssi)) => (
            "wifi-ok",
            format!("Good signal ({} dBm).", rssi),
        ),
        (AudioStatus::Ready, WifiStatus::Strong(rssi)) => (
            "wifi-ok",
            format!("Strong signal ({} dBm).", rssi),
        ),
    }
}

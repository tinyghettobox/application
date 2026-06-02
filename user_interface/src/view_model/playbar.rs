use crate::{model::{Field, State}, view_model::update_ui, AppWindow, Playbar};
use slint::{ComponentHandle, ModelRc, VecModel, Weak};

pub struct PlaybarVM {
    ui: Weak<AppWindow>,
    state: State,
}

impl PlaybarVM {
    pub fn new(ui: Weak<AppWindow>, state: State) -> Self {
        let vm = PlaybarVM { ui, state };
        vm.setup_ui();
        vm.setup_state_listeners();
        vm
    }

    fn setup_ui(&self) {
        if let Some(ui) = self.ui.upgrade() {
            let playbar = ui.global::<Playbar>();

            {
                let state = self.state.clone();
                playbar.on_toggle_play(move || {
                    let is_playing = state.is_playing();
                    state.dispatch(crate::model::actions::Action::TogglePlay(!is_playing));
                });
            }

            {
                let state = self.state.clone();
                playbar.on_play_prev(move || {
                    state.dispatch(crate::model::actions::Action::PlayPrev);
                });
            }

            {
                let state = self.state.clone();
                playbar.on_play_next(move || {
                    state.dispatch(crate::model::actions::Action::PlayNext);
                });
            }

            {
                let state = self.state.clone();
                playbar.on_set_volume(move |volume| {
                    state.dispatch(crate::model::actions::Action::SetVolume(volume));
                });
            }

            {
                let state = self.state.clone();
                playbar.on_seek(move |pct| {
                    state.dispatch(crate::model::actions::Action::SeekTo(pct as f64));
                });
            }
        }
    }

    fn setup_state_listeners(&self) {
        let ui_weak = self.ui.clone();
        let state = self.state.clone();

        self.state.subscribe(move |changes| {
            let relevant = changes.iter().any(|f| {
                matches!(
                    f,
                    Field::playing_library_entry(_)
                        | Field::is_playing(_)
                        | Field::is_loading(_)
                        | Field::progress(_)
                        | Field::volume(_)
                )
            });
            if !relevant {
                return;
            }

            let playing = state.playing_library_entry();
            let is_playing = state.is_playing();
            let is_loading = state.is_loading();
            let progress = state.progress().as_f64() as f32;
            let volume = state.volume();

            update_ui(&ui_weak, move |ui| {
                let playbar = ui.global::<Playbar>();

                playbar.set_visible(playing.is_some() || is_loading);
                playbar.set_is_loading(is_loading);
                playbar.set_progress(progress);
                playbar.set_is_playing(is_playing);
                playbar.set_volume(volume);

                if let Some(entry) = playing {
                    playbar.set_track_title(entry.name.clone().into());
                    playbar.set_parent_title(
                        entry.parent_name.clone().unwrap_or_default().into(),
                    );

                    let track_image = entry
                        .image
                        .clone()
                        .map(|img| {
                            ModelRc::new(VecModel::from(
                                img.into_iter().map(|b| b as i32).collect::<Vec<_>>(),
                            ))
                        })
                        .unwrap_or_else(|| ModelRc::new(VecModel::default()));

                    let parent_image = entry
                        .parent_image
                        .clone()
                        .map(|img| {
                            ModelRc::new(VecModel::from(
                                img.into_iter().map(|b| b as i32).collect::<Vec<_>>(),
                            ))
                        })
                        .unwrap_or_else(|| ModelRc::new(VecModel::default()));

                    playbar.set_track_image(track_image);
                    playbar.set_parent_image(parent_image);
                }
            });
        });
    }
}

impl Clone for PlaybarVM {
    fn clone(&self) -> Self {
        PlaybarVM {
            ui: self.ui.clone(),
            state: self.state.clone(),
        }
    }
}

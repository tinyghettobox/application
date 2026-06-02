use std::f32::consts::PI;

use crate::{model::{Field, State}, view_model::update_ui, AppWindow, Content};
use database::model::library_entry::{Model as LibraryEntry, Variant};
use slint::{ComponentHandle, ModelRc, VecModel, Weak};

pub struct ContentVM {
    ui: Weak<AppWindow>,
    state: State,
}

impl ContentVM {
    pub fn new(ui: Weak<AppWindow>, state: State) -> Self {
        let vm = ContentVM { ui, state };
        vm.setup_ui();
        vm.setup_state_listeners();
        vm
    }
 
    pub fn setup_ui(&self) {
        if let Some(ui) = self.ui.upgrade() {
            let content = ui.global::<Content>();

            {
                let state_ = self.state.clone();
                content.on_select_library_entry(move |id| {
                    let entry = state_
                        .active_library_entry()
                        .and_then(|e| e.children)
                        .and_then(|children| children.into_iter().find(|e| e.id == id));

                    match entry {
                        Some(e) if e.variant != database::model::library_entry::Variant::Folder => {
                            state_.dispatch(crate::model::actions::Action::StartPlayingLibraryEntry(e));
                        }
                        _ => {
                            state_.dispatch(crate::model::actions::Action::LoadLibraryEntry(id));
                        }
                    }
                });
            }

            {
                let state_ = self.state.clone();
                content.on_play_folder(move |id| {
                    let entry = state_
                        .active_library_entry()
                        .and_then(|e| e.children)
                        .and_then(|children| children.into_iter().find(|e| e.id == id));

                    if let Some(e) = entry {
                        state_.dispatch(crate::model::actions::Action::StartPlayingLibraryEntry(e));
                    }
                });
            }

            {
                let state_ = self.state.clone();
                content.on_context_menu_action(move |id, action| {
                    match action.as_str() {
                        "played" => state_.dispatch(crate::model::actions::Action::SetPlayedAt(id, true)),
                        "not-played" => state_.dispatch(crate::model::actions::Action::SetPlayedAt(id, false)),
                        _ => {}
                    }
                });
            }
        }
    }

    pub fn setup_state_listeners(&self) {
        // Clone for move closure
        let ui_weak = self.ui.clone();
        let state = self.state.clone();
        
        self.state.subscribe(move |changes| {
            let has_active = changes.iter().any(|f| matches!(f, Field::active_library_entry(_)));
            let has_loading = changes.iter().any(|f| matches!(f, Field::is_loading(_)));
            let has_playing = changes.iter().any(|f| matches!(f, Field::playing_ancestor_ids(_)));

            if has_loading {
                let is_loading = state.is_loading();
                let ui_weak2 = ui_weak.clone();
                update_ui(&ui_weak2, move |ui| {
                    ui.global::<Content>().set_interaction_disabled(is_loading);
                });
            }

            if !has_active && !has_playing {
                return;
            }

            let entry = match state.active_library_entry() {
                Some(e) => e,
                None => return,
            };
            let children = match entry.children {
                Some(c) => c,
                None => return,
            };

            let state = state.clone();
            update_ui(&ui_weak, move |ui| {
                let content = ui.global::<Content>();
                let first_variant = children.first().map(|e| e.variant.clone());
                let variant_str = children.first().map(|e| e.variant.to_string()).unwrap_or_default();
                content.set_variant(variant_str.as_str().into());

                let is_tile = matches!(first_variant, Some(Variant::Folder) | Some(Variant::Stream));
                if is_tile {
                    Self::set_tile_view_data(&content, children, state);
                } else {
                    Self::set_detail_view_data(&content, children, state);
                }
            });
        });
    }

    fn set_tile_view_data(ui: &Content<'_>, entries: Vec<LibraryEntry>, state: State) {
        let rows_data: Vec<Vec<_>> = entries
            .chunks(3)
            .map(|chunk| {
                chunk.iter().map(|entry| Self::map_library_entry_to_ui(entry, &state)).collect()
            })
            .collect();

        let rows_model = ModelRc::new(VecModel::from(
            rows_data
                .into_iter()
                .map(|row| ModelRc::new(VecModel::from(row)))
                .collect::<Vec<_>>(),
        ));

        ui.set_tile_rows(rows_model);
        ui.set_detail_rows(ModelRc::default());
    }

    fn set_detail_view_data(ui: &Content<'_>, entries: Vec<LibraryEntry>, state: State) {
        let rows_data: Vec<_> = entries
            .iter()
            .map(|entry| Self::map_library_entry_to_ui(entry, &state))
            .collect();

        let rows_model = ModelRc::new(VecModel::from(rows_data));

        ui.set_tile_rows(ModelRc::default());
        ui.set_detail_rows(rows_model);
    }

    fn map_library_entry_to_ui(entry: &LibraryEntry, state: &State) -> crate::UILibraryEntry {
        let playing = state.playing_library_entry();
        let is_exact_match = playing.as_ref().map_or(false, |p| p.id == entry.id);
        let is_ancestor = state.playing_ancestor_ids().contains(&entry.id);

        let play_progress = entry.play_progress.unwrap_or(0);
        // Arc endpoint for a circle of radius 90 centered at (90,90), starting from top, clockwise
        let angle = (play_progress as f32 / 100.0) * 2.0 * PI;
        let progress_arc_x = 90.0 + 90.0 * angle.sin();
        let progress_arc_y = 90.0 - 90.0 * angle.cos();

        crate::UILibraryEntry {
            id: entry.id,
            parent_id: entry.parent_id.unwrap_or(0),
            variant: entry.variant.to_string().into(),
            name: entry.name.clone().into(),
            played_at: entry.played_at
                .map(|dt| dt.format("%Y-%m-%d").to_string())
                .unwrap_or_default()
                .into(),
            image: entry.image.clone()
                .map(|img| ModelRc::new(VecModel::from(
                    img.into_iter().map(|b| b as i32).collect::<Vec<_>>(),
                )))
                .unwrap_or_else(|| ModelRc::new(VecModel::default())),
            sort_key: entry.sort_key,
            is_playing: is_exact_match && state.is_playing(),
            is_loaded: is_exact_match || is_ancestor,
            play_progress,
            progress_arc_x,
            progress_arc_y,
        }
    }
}

impl Clone for ContentVM {
    fn clone(&self) -> Self {
        ContentVM {
            ui: self.ui.clone(),
            state: self.state.clone(),
        }
    }
}
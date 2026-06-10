use std::f32::consts::PI;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::{model::{Field, State}, view_model::update_ui, AppWindow, Content};
use database::model::library_entry::{Model as LibraryEntry, Variant};
use slint::{ComponentHandle, ModelRc, VecModel, Weak};

// SYNC: must match `row-height` property in tile_list_view.slint
// Tile VerticalBox: padding(8+8) + ring(180) + spacing(10) + text(58) = 264px, + row-gap(8) = 272px
const ROW_HEIGHT_PX: f32 = 272.0;
const DISPLAY_HEIGHT_PX: f32 = 480.0;
/// Extra rows rendered above and below the visible area to prevent pop-in during fast scrolls.
const ROW_BUFFER: usize = 2;

pub struct ContentVM {
    ui: Weak<AppWindow>,
    state: State,
    /// All tile entries for the active folder. Rust owns them; Slint only sees the visible window.
    tile_store: Arc<Mutex<Option<Vec<LibraryEntry>>>>,
    /// Cached first-row index of the last rendered window. Guards against redundant rebuilds.
    tile_window_first_row: Arc<Mutex<usize>>,
    /// Saved scroll positions (viewport-y, always ≤ 0) per folder id.
    scroll_positions: Arc<Mutex<HashMap<i32, f32>>>,
    prev_folder_id: Arc<Mutex<Option<i32>>>,
    is_prev_tile: Arc<Mutex<bool>>,
}

impl ContentVM {
    pub fn new(ui: Weak<AppWindow>, state: State) -> Self {
        let vm = ContentVM {
            ui,
            state,
            tile_store: Arc::new(Mutex::new(None)),
            tile_window_first_row: Arc::new(Mutex::new(0)),
            scroll_positions: Arc::new(Mutex::new(HashMap::new())),
            prev_folder_id: Arc::new(Mutex::new(None)),
            is_prev_tile: Arc::new(Mutex::new(false)),
        };
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
                        .and_then(|children| children.into_iter().find(|e| e.id == id && !e.deleted));

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
                        .and_then(|children| children.into_iter().find(|e| e.id == id && !e.deleted));

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

            // Rebuild the visible window whenever the user scrolls past a row boundary.
            {
                let tile_store = self.tile_store.clone();
                let tile_window_first_row = self.tile_window_first_row.clone();
                let state_ = self.state.clone();
                let ui_weak = self.ui.clone();
                content.on_tile_scroll_changed(move |scroll_y| {
                    let store_guard = tile_store.lock().unwrap();
                    if let Some(entries) = store_guard.as_ref() {
                        if let Some(ui) = ui_weak.upgrade() {
                            let mut cached = tile_window_first_row.lock().unwrap();
                            Self::update_visible_window(
                                &ui.global::<Content>(),
                                entries,
                                scroll_y,
                                &mut cached,
                                &state_,
                                false,
                            );
                        }
                    }
                });
            }
        }
    }

    pub fn setup_state_listeners(&self) {
        let ui_weak = self.ui.clone();
        let state = self.state.clone();
        let tile_store = self.tile_store.clone();
        let tile_window_first_row = self.tile_window_first_row.clone();
        let scroll_positions = self.scroll_positions.clone();
        let prev_folder_id = self.prev_folder_id.clone();
        let is_prev_tile = self.is_prev_tile.clone();

        self.state.subscribe(move |changes| {
            let has_active  = changes.iter().any(|f| matches!(f, Field::active_library_entry(_)));
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
                Some(c) => c.into_iter().filter(|e| !e.deleted).collect::<Vec<_>>(),
                None => return,
            };

            let state = state.clone();
            let tile_store = tile_store.clone();
            let tile_window_first_row = tile_window_first_row.clone();
            let scroll_positions = scroll_positions.clone();
            let prev_folder_id = prev_folder_id.clone();
            let is_prev_tile = is_prev_tile.clone();
            let entry_id = entry.id;
            update_ui(&ui_weak, move |ui| {
                let content = ui.global::<Content>();
                let first_variant = children.first().map(|e| e.variant.clone());
                let variant_str = children.first().map(|e| e.variant.to_string()).unwrap_or_default();
                content.set_variant(variant_str.as_str().into());

                let is_tile = matches!(first_variant, Some(Variant::Folder) | Some(Variant::Stream));

                if has_active && *is_prev_tile.lock().unwrap() {
                    // Save the scroll position of the folder we are navigating away from.
                    if let Some(old_id) = *prev_folder_id.lock().unwrap() {
                        scroll_positions.lock().unwrap().insert(old_id, content.get_tile_scroll_y());
                    }
                }

                if is_tile {
                    let scroll_y = if has_active {
                        scroll_positions.lock().unwrap().get(&entry_id).copied().unwrap_or(0.0)
                    } else {
                        content.get_tile_scroll_y() // play-state update: keep current position
                    };

                    // Hand all entries to the Rust store; Slint only ever sees the visible window.
                    *tile_store.lock().unwrap() = Some(children);
                    content.set_detail_rows(ModelRc::default());

                    {
                        let store_guard = tile_store.lock().unwrap();
                        let entries = store_guard.as_ref().unwrap();
                        let mut cached = tile_window_first_row.lock().unwrap();
                        Self::update_visible_window(&content, entries, scroll_y, &mut cached, &state, true);
                    }

                    if has_active {
                        // viewport-height is now deterministic (total_rows * constant), so restore is immediate.
                        content.set_tile_restore_scroll_y(scroll_y);
                        content.set_tile_restore_scroll_seq(content.get_tile_restore_scroll_seq() + 1);
                        *prev_folder_id.lock().unwrap() = Some(entry_id);
                        *is_prev_tile.lock().unwrap() = true;
                    }
                } else {
                    *tile_store.lock().unwrap() = None;
                    Self::set_detail_view_data(&content, children, &state);
                    if has_active {
                        *prev_folder_id.lock().unwrap() = Some(entry_id);
                        *is_prev_tile.lock().unwrap() = false;
                    }
                }
            });
        });
    }

    /// Compute the visible window of rows from `scroll_y` and push it to Slint.
    ///
    /// `force` — when `false`, skips the rebuild if `first_row` hasn't changed (optimization for
    /// scroll events that don't cross a row boundary). Pass `true` on navigation and play-state
    /// updates where data may have changed even if the row index is the same.
    fn update_visible_window(
        ui: &Content<'_>,
        entries: &[LibraryEntry],
        scroll_y: f32,
        cached_first_row: &mut usize,
        state: &State,
        force: bool,
    ) {
        let total_rows = (entries.len() + 2) / 3;
        ui.set_tile_total_row_count(total_rows as i32);

        let scroll_depth = (-scroll_y).max(0.0);
        let first_row = ((scroll_depth / ROW_HEIGHT_PX) as usize).saturating_sub(ROW_BUFFER);
        let last_row = (((scroll_depth + DISPLAY_HEIGHT_PX) / ROW_HEIGHT_PX).ceil() as usize + ROW_BUFFER)
            .min(total_rows);

        if !force && first_row == *cached_first_row {
            return;
        }
        *cached_first_row = first_row;

        let rows_model = ModelRc::new(VecModel::from(
            entries.chunks(3)
                .skip(first_row)
                .take(last_row.saturating_sub(first_row))
                .map(|chunk| ModelRc::new(VecModel::from(
                    chunk.iter().map(|e| Self::map_library_entry_to_ui(e, state)).collect::<Vec<_>>()
                )))
                .collect::<Vec<_>>()
        ));
        ui.set_tile_first_row(first_row as i32);
        ui.set_tile_rows(rows_model);
    }

    fn set_detail_view_data(ui: &Content<'_>, entries: Vec<LibraryEntry>, state: &State) {
        ui.set_tile_rows(ModelRc::default());
        ui.set_tile_total_row_count(0);
        ui.set_tile_first_row(0);
        ui.set_detail_rows(ModelRc::new(VecModel::from(
            entries.iter().map(|e| Self::map_library_entry_to_ui(e, state)).collect::<Vec<_>>()
        )));
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
            tile_store: self.tile_store.clone(),
            tile_window_first_row: self.tile_window_first_row.clone(),
            scroll_positions: self.scroll_positions.clone(),
            prev_folder_id: self.prev_folder_id.clone(),
            is_prev_tile: self.is_prev_tile.clone(),
        }
    }
}
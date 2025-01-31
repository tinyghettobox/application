use std::sync::{Arc, Mutex};

use crate::components::tile_list_item::widget::TileListItemWidget;
use crate::components::{Children, Component};
use crate::state::{Action, Dispatcher, Event, EventHandler, State};
use database::model::library_entry::Variant;
use tracing::{debug, error};

pub struct TileListItemComponent {
    pub widget: TileListItemWidget,
    pub children: Vec<Arc<Mutex<Box<dyn EventHandler>>>>,
    state: Arc<Mutex<State>>,
    library_entry_id: i32,
}

impl EventHandler for TileListItemComponent {
    fn on_event(&mut self, _event: &Event) {}

    fn get_children(&self) -> Vec<Arc<Mutex<Box<dyn EventHandler>>>> {
        self.children.clone()
    }
}

impl Component<i32> for TileListItemComponent {
    fn new(state: Arc<Mutex<State>>, dispatcher: Arc<Mutex<Dispatcher>>, library_entry_id: i32) -> Self {
        let (widget, children) = Self::render(state.clone(), dispatcher.clone(), library_entry_id);
        let mut component = Self {
            widget,
            state,
            children,
            library_entry_id,
        };
        component.update();
        component
    }

    #[allow(refining_impl_trait)]
    fn render(
        state: Arc<Mutex<State>>,
        dispatcher: Arc<Mutex<Dispatcher>>,
        library_entry_id: i32,
    ) -> (TileListItemWidget, Children) {
        let widget = TileListItemWidget::new();

        {
            let state = state.clone();
            let dispatcher = dispatcher.clone();
            widget.connect_clicked(move || {
                let library_entry = state.lock().unwrap().library_entry.children.as_ref().and_then(|children| {
                    children.iter().find(|library_entry| library_entry.id == library_entry_id).cloned()
                });
                // We use tile list component also for stream list to show them with an image. Thus playing the stream instead of selecting
                if let Some(library_entry) = library_entry {
                    if let Variant::Stream = library_entry.variant {
                        dispatcher
                            .lock()
                            .unwrap()
                            .dispatch_action(Action::Play(library_entry.parent_id.unwrap(), Some(library_entry.id)));
                        return;
                    }
                }
                dispatcher.lock().unwrap().dispatch_action(Action::Select(library_entry_id));
            });
        }
        {
            let dispatcher = dispatcher.clone();
            let state = state.clone();
            widget.connect_play_clicked(move || {
                let library_entry = state.lock().unwrap().library_entry.children.as_ref().and_then(|children| {
                    children.iter().find(|library_entry| library_entry.id == library_entry_id).cloned()
                });

                if let Some(library_entry) = library_entry {
                    if let Variant::Stream = library_entry.variant {
                        dispatcher
                            .lock()
                            .unwrap()
                            .dispatch_action(Action::Play(library_entry.parent_id.unwrap(), Some(library_entry.id)));
                    } else {
                        dispatcher.lock().unwrap().dispatch_action(Action::Play(library_entry.id, None));
                    }
                }
            });
        }

        (widget, vec![])
    }

    fn update(&mut self) {
        let state = self.state.lock().unwrap();

        match &state.library_entry.children {
            Some(child_library_entry) => {
                debug!(
                    "Searching id {} in {} children",
                    self.library_entry_id,
                    child_library_entry.len()
                );
                match child_library_entry.iter().find(|entry| entry.id == self.library_entry_id) {
                    Some(entry) => {
                        debug!(
                            "Entry {} has image size: {}",
                            entry.id,
                            entry.image.as_ref().unwrap_or(&vec![]).len()
                        );
                        self.widget.set_image(entry.image.clone());
                        self.widget.set_name(entry.name.to_string());
                    }
                    None => error!("Passed library entry '{}' does not exist o.O???", self.library_entry_id),
                }
            }
            None => error!("Library entry has no children o.O???"),
        }
    }

    #[allow(refining_impl_trait)]
    fn get_widget(&self) -> TileListItemWidget {
        self.widget.clone()
    }
}

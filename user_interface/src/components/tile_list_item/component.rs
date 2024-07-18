use std::sync::{Arc};
use std::sync::Mutex;
use gtk4::prelude::IsA;
use gtk4::Widget;
use tracing::{debug, error, info};
use crate::components::{Children, Component};
use crate::components::tile_list_item::widget::TileListItemWidget;
use crate::state::{Action, Dispatcher, Event, EventHandler, State};

pub struct TileListItemComponent {
    pub widget: TileListItemWidget,
    pub children: Vec<Arc<Mutex<Box<dyn EventHandler>>>>,
    state: Arc<Mutex<State>>,
    library_entry_id: i32,
}

impl EventHandler for TileListItemComponent {
    fn on_event(&mut self, _event: &Event) {

    }

    fn get_children(&self) -> Vec<Arc<Mutex<Box<dyn EventHandler>>>> {
        self.children.clone()
    }
}

impl Component<i32> for TileListItemComponent {
    fn new(state: Arc<Mutex<State>>, dispatcher: Arc<Mutex<Dispatcher>>, library_entry_id: i32) -> Self {
        let (widget, children) = Self::render(state.clone(), dispatcher.clone(), library_entry_id);
        let mut component = Self { widget, state, children, library_entry_id };
        component.update();
        component
    }

    fn render(_state: Arc<Mutex<State>>, dispatcher: Arc<Mutex<Dispatcher>>, library_entry_id: i32) -> (TileListItemWidget, Children) {
        let widget = TileListItemWidget::new();

        let dispatcher = dispatcher.clone();
        widget.connect_clicked(move || {
            debug!("connect_clicked start");
            dispatcher
                .lock()
                .expect("could not lock dispatcher")
                .dispatch_action(Action::Select(library_entry_id));
            debug!("connect_clicked end");
        });

        (widget, vec![])
    }

    fn update(&mut self) {
        debug!("tile_list_item update start");
        let state = self.state.lock().expect("Could not lock state");

        match &state.library_entry.children {
            Some(child_library_entry) => {
                info!("Searching id {} in {} children", self.library_entry_id, child_library_entry.len());
                match child_library_entry.iter().find(|entry| entry.id == self.library_entry_id) {
                    Some(entry) => {
                        info!("Entry {} has image size: {}", entry.id, entry.image.as_ref().unwrap_or(&vec![]).len());
                        self.widget.set_image(entry.image.clone());
                        self.widget.set_name(entry.name.to_string());
                    },
                    None => error!("Passed library entry '{}' does not exist o.O???", self.library_entry_id)
                }
            }
            None => error!("Library entry has no children o.O???")
        }
        debug!("tile_list_item update end");
    }

    fn get_widget(&self) -> impl IsA<Widget> {
        self.widget.clone()
    }
}
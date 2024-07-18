use std::sync::{Arc};
use std::sync::Mutex;

use gtk4::prelude::{IsA};
use gtk4::Widget;
use tracing::{debug, info};

use crate::components::{Children, Component};
use crate::components::tile_list::widget::TileListWidget;
use crate::components::tile_list_item::TileListItemComponent;
use crate::state::{Dispatcher, Event, EventHandler, State};

pub struct TileListComponent {
    pub widget: TileListWidget,
    pub children: Vec<Arc<Mutex<Box<dyn EventHandler>>>>,
    state: Arc<Mutex<State>>,
    dispatcher: Arc<Mutex<Dispatcher>>
}

impl EventHandler for TileListComponent {
    fn on_event(&mut self, event: &Event) {
        match event {
            Event::LibraryEntryChanged => self.update(),
            _ => {}
        }
    }

    fn get_children(&self) -> Vec<Arc<Mutex<Box<dyn EventHandler>>>> {
        self.children.clone()
    }
}

impl Component<Option<()>> for TileListComponent {
    fn new(state: Arc<Mutex<State>>, dispatcher: Arc<Mutex<Dispatcher>>, params: Option<()>) -> Self {
        let (widget, children) = Self::render(state.clone(), dispatcher.clone(), params);
        let mut component = Self { widget, state, dispatcher, children };
        component.update();
        component
    }

    fn render(_state: Arc<Mutex<State>>, _dispatcher: Arc<Mutex<Dispatcher>>, _params: Option<()>) -> (TileListWidget, Children) {
        let widget = TileListWidget::new();

        (widget, vec![])
    }

    fn update(&mut self) {
        debug!("tile_list start");
        if self.state.lock().expect("could not lock state").active_view != "tile_list" {
            debug!("tile_list return");
            return;
        }
        let library_entry_ids = self.state
            .lock()
            .expect("could not lock state")
            .library_entry
            .children
            .as_ref()
            .map(|children| children.iter().map(|entry| entry.id).collect::<Vec<i32>>());

        match library_entry_ids {
            Some(library_entry_ids) => {
                info!("Creating {} tiles", library_entry_ids.len());
                let child_components = library_entry_ids.into_iter()
                    .map(|library_entry_id| TileListItemComponent::new(self.state.clone(), self.dispatcher.clone(), library_entry_id))
                    .collect::<Vec<TileListItemComponent>>();

                self.widget.remove_children();
                self.widget.set_children(&child_components);

                child_components.into_iter().for_each(|comp| self.children.push(Arc::new(Mutex::new(Box::new(comp)))));

                info!("done")
            },
            None => {
                info!("No children, no tiles")
                // TODO show label about empty children
            }
        }
        debug!("tile_list end");
    }

    fn get_widget(&self) -> impl IsA<Widget> {
        self.widget.clone()
    }
}

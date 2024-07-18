use std::sync::{Arc};
use std::sync::Mutex;
use gtk4::glib::BoxedAnyObject;

use gtk4::prelude::{IsA};
use gtk4::{gio, Widget};
use tracing::{debug, error};

use crate::components::{Children, Component};
use crate::components::detail_list::widget::DetailListWidget;
use crate::state::{Dispatcher, Event, EventHandler, State};

pub struct DetailListComponent {
    pub widget: DetailListWidget,
    pub children: Vec<Arc<Mutex<Box<dyn EventHandler>>>>,
    state: Arc<Mutex<State>>,
}

impl EventHandler for DetailListComponent {
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

impl Component<Option<()>> for DetailListComponent {
    fn new(state: Arc<Mutex<State>>, dispatcher: Arc<Mutex<Dispatcher>>, params: Option<()>) -> Self {
        let (widget, children) = Self::render(state.clone(), dispatcher.clone(), params);
        let mut component = Self { widget, state, children };
        component.update();
        component
    }

    fn render(state: Arc<Mutex<State>>, dispatcher: Arc<Mutex<Dispatcher>>, _params: Option<()>) -> (DetailListWidget, Children) {
        let widget = DetailListWidget::new(state.clone(), dispatcher.clone());

        (widget, vec![])
    }

    fn update(&mut self) {
        debug!("update detail_list");
        if self.state.lock().expect("could not lock state").active_view != "detail_list" {
            debug!("update detail_list return");
            return;
        }
        let list_store = self.state.lock().expect("could not lock state")
            .library_entry
            .children
            .clone()
            .map(|children| {
                children
                    .into_iter()
                    .map(|child| BoxedAnyObject::new(child))
                    .collect::<gio::ListStore>()
            });

        match list_store {
            Some(list_store) => {
                debug!("Setting list store with items");
                self.widget.set_list_store(list_store);
            },
            None => {
                error!("Want to render detail list but no children are available? o.O");
            }
        }

        debug!("update detail_list done");
    }

    fn get_widget(&self) -> impl IsA<Widget> {
        self.widget.clone()
    }
}

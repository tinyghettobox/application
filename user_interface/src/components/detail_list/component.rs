use gtk4::glib::BoxedAnyObject;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use gtk4::gio;
use tracing::error;

use crate::components::detail_list::widget::DetailListWidget;
use crate::components::{Children, Component};
use crate::state::{Dispatcher, Event, EventHandler, State};

pub struct DetailListComponent {
    pub widget: DetailListWidget,
    pub children: Rc<RefCell<Vec<Arc<Mutex<Box<dyn EventHandler>>>>>>,
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
        self.children.borrow().clone()
    }
}

impl Component<Option<()>> for DetailListComponent {
    fn new(state: Arc<Mutex<State>>, dispatcher: Arc<Mutex<Dispatcher>>, params: Option<()>) -> Self {
        let children = Rc::new(RefCell::new(vec![]));
        let (widget, _) = Self::render(state.clone(), dispatcher.clone(), params);
        widget.set_factory(state.clone(), dispatcher.clone(), children.clone());

        let mut component = Self {
            widget,
            state,
            children,
        };
        component.update();
        component
    }

    #[allow(refining_impl_trait)]
    fn render(
        _state: Arc<Mutex<State>>,
        _dispatcher: Arc<Mutex<Dispatcher>>,
        _params: Option<()>,
    ) -> (DetailListWidget, Children) {
        let widget = DetailListWidget::new();
        (widget, vec![])
    }

    fn update(&mut self) {
        if self.state.lock().unwrap().active_view != "detail_list" {
            return;
        }
        let list_store =
            self.state.lock().unwrap().library_entry.children.clone().map(|children| {
                children.into_iter().map(|child| BoxedAnyObject::new(child)).collect::<gio::ListStore>()
            });

        match list_store {
            Some(list_store) => {
                self.widget.set_list_store(list_store);
            }
            None => {
                error!("Want to render detail list but no children are available? o.O");
            }
        }
    }

    #[allow(refining_impl_trait)]
    fn get_widget(&self) -> DetailListWidget {
        self.widget.clone()
    }
}

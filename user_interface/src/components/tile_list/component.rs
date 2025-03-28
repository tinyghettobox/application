use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use tracing::error;

use crate::components::tile_list::widget::TileListWidget;
use crate::components::tile_list_item::TileListItemComponent;
use crate::components::{Children, Component};
use crate::state::{Dispatcher, Event, EventHandler, State};

#[derive(Clone)]
pub struct TileListComponent {
    pub widget: TileListWidget,
    pub children: Rc<RefCell<Vec<Arc<Mutex<Box<dyn EventHandler>>>>>>,
    state: Arc<Mutex<State>>,
    dispatcher: Arc<Mutex<Dispatcher>>,
}

impl EventHandler for TileListComponent {
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

impl Component<Option<()>> for TileListComponent {
    fn new(
        state: Arc<Mutex<State>>,
        dispatcher: Arc<Mutex<Dispatcher>>,
        params: Option<()>,
    ) -> Self {
        let children = Rc::new(RefCell::new(vec![]));
        let (widget, _) = Self::render(state.clone(), dispatcher.clone(), params);
        let mut component = Self {
            widget,
            state,
            dispatcher,
            children,
        };
        component.update();
        component.start_lazy_loading();
        component
    }

    #[allow(refining_impl_trait)]
    fn render(
        _state: Arc<Mutex<State>>,
        _dispatcher: Arc<Mutex<Dispatcher>>,
        _params: Option<()>,
    ) -> (TileListWidget, Children) {
        let widget = TileListWidget::new();

        (widget, vec![])
    }

    fn update(&mut self) {
        if self.state.lock().unwrap().active_view != "tile_list" {
            return;
        }
        self.widget.remove_children();
        self.children.borrow_mut().clear();
        self.append_list_items(0, 12);
    }

    #[allow(refining_impl_trait)]
    fn get_widget(&self) -> TileListWidget {
        self.widget.clone()
    }
}

impl TileListComponent {
    fn start_lazy_loading(&self) {
        let self_ = self.clone();

        self.widget.connect_scroll_end(move || {
            let mut self_ = self_.clone();
            let child_amount = self_.get_children().len();
            self_.append_list_items(child_amount, 6);
        });
    }

    fn append_list_items(&mut self, start_index: usize, amount: usize) {
        let library_entry_ids = self
            .state
            .lock()
            .unwrap()
            .library_entry
            .children
            .as_ref()
            .map(|children| {
                children
                    .iter()
                    .skip(start_index)
                    .take(amount)
                    .map(|entry| entry.id)
                    .collect::<Vec<i32>>()
            });

        match library_entry_ids {
            Some(library_entry_ids) => {
                let child_components = library_entry_ids
                    .into_iter()
                    .map(|library_entry_id| {
                        TileListItemComponent::new(
                            self.state.clone(),
                            self.dispatcher.clone(),
                            library_entry_id,
                        )
                    })
                    .collect::<Vec<TileListItemComponent>>();

                let mut child_widgets = vec![];
                for comp in &child_components {
                    child_widgets.push(comp.get_widget());
                }

                self.widget
                    .set_children(&child_widgets, start_index as i32 / 3, 0);

                for child in child_components.into_iter() {
                    self.children
                        .borrow_mut()
                        .push(Arc::new(Mutex::new(Box::new(child))))
                }
            }
            None => {
                error!("Tile list should only be rendered with children")
            }
        }
    }
}

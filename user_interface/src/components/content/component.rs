use crate::components::content::widget::ContentWidget;
use crate::components::detail_list::DetailListComponent;
use crate::components::empty_info::EmptyInfoComponent;
use crate::components::tile_list::TileListComponent;
use crate::components::{Children, Component};
use crate::state::{Dispatcher, Event, EventHandler, State};
use std::sync::{Arc, Mutex};

pub struct ContentComponent {
    pub widget: ContentWidget,
    pub children: Vec<Arc<Mutex<Box<dyn EventHandler>>>>,
    state: Arc<Mutex<State>>,
}

impl EventHandler for ContentComponent {
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

impl Component<Option<()>> for ContentComponent {
    fn new(state: Arc<Mutex<State>>, dispatcher: Arc<Mutex<Dispatcher>>, params: Option<()>) -> Self {
        let (widget, children) = Self::render(state.clone(), dispatcher.clone(), params);
        let mut component = ContentComponent {
            state,
            widget,
            children,
        };
        component.update();
        component
    }

    #[allow(refining_impl_trait)]
    fn render(
        state: Arc<Mutex<State>>,
        dispatcher: Arc<Mutex<Dispatcher>>,
        _params: Option<()>,
    ) -> (ContentWidget, Children) {
        let widget = ContentWidget::new();

        let tile_list = TileListComponent::new(state.clone(), dispatcher.clone(), None);
        let detail_list = DetailListComponent::new(state.clone(), dispatcher.clone(), None);
        let empty_info = EmptyInfoComponent::new(state.clone(), dispatcher.clone(), None);

        widget.append_child("tile_list", &tile_list.get_widget());
        widget.append_child("detail_list", &detail_list.get_widget());
        widget.append_child("empty_info", &empty_info.get_widget());

        (
            widget,
            vec![
                Arc::new(Mutex::new(Box::new(tile_list))),
                Arc::new(Mutex::new(Box::new(detail_list))),
                Arc::new(Mutex::new(Box::new(empty_info))),
            ],
        )
    }

    fn update(&mut self) {
        let state = self.state.lock().unwrap();
        self.widget.set_active_child(state.active_view.to_owned());
    }

    #[allow(refining_impl_trait)]
    fn get_widget(&self) -> ContentWidget {
        self.widget.clone()
    }
}

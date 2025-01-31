use crate::components::empty_info::widget::EmptyInfoWidget;
use crate::components::{Children, Component};
use crate::state::{Dispatcher, Event, EventHandler, State};
use std::sync::{Arc, Mutex};

pub struct EmptyInfoComponent {
    pub widget: EmptyInfoWidget,
    pub children: Vec<Arc<Mutex<Box<dyn EventHandler>>>>,
}

impl EventHandler for EmptyInfoComponent {
    fn on_event(&mut self, _event: &Event) {}

    fn get_children(&self) -> Vec<Arc<Mutex<Box<dyn EventHandler>>>> {
        self.children.clone()
    }
}

impl Component<Option<()>> for EmptyInfoComponent {
    fn new(state: Arc<Mutex<State>>, dispatcher: Arc<Mutex<Dispatcher>>, params: Option<()>) -> Self {
        let (widget, children) = Self::render(state, dispatcher, params);
        Self { widget, children }
    }

    #[allow(refining_impl_trait)]
    fn render(
        _state: Arc<Mutex<State>>,
        _dispatcher: Arc<Mutex<Dispatcher>>,
        _params: Option<()>,
    ) -> (EmptyInfoWidget, Children) {
        (EmptyInfoWidget::new(), vec![])
    }

    fn update(&mut self) {}

    #[allow(refining_impl_trait)]
    fn get_widget(&self) -> EmptyInfoWidget {
        self.widget.clone()
    }
}

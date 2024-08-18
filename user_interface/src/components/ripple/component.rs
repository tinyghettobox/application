use std::sync::{Arc, Mutex};

use gtk4::prelude::*;
use gtk4::Widget;

use crate::components::{Children, Component};
use crate::components::ripple::widget::RippleWidget;
use crate::state::{Dispatcher, Event, EventHandler, State};

pub struct RippleComponent {
    pub widget: RippleWidget,
    pub children: Vec<Arc<Mutex<Box<dyn EventHandler>>>>,
}


impl EventHandler for RippleComponent {
    fn on_event(&mut self, _event: &Event) {}

    fn get_children(&self) -> Vec<Arc<Mutex<Box<dyn EventHandler>>>> {
        self.children.clone()
    }
}

impl Component<Option<()>> for RippleComponent {
    fn new(state: Arc<Mutex<State>>, dispatcher: Arc<Mutex<Dispatcher>>, params: Option<()>) -> Self {
        let (widget, children) = Self::render(state.clone(), dispatcher.clone(), params);
        let mut component = Self { widget, children };
        component.update();
        component
    }

    #[allow(refining_impl_trait)]
    fn render(_state: Arc<Mutex<State>>, _dispatcher: Arc<Mutex<Dispatcher>>, _params: Option<()>) -> (RippleWidget, Children) {
        let widget = RippleWidget::new();
        widget.start_ripple_drawing();

        (widget, vec![])
    }

    fn update(&mut self) {}

    fn get_widget(&self) -> impl IsA<Widget> {
        self.widget.clone()
    }
}

impl RippleComponent {
    pub fn add_child(&self, widget: &impl IsA<Widget>) {
        self.widget.add_child(widget);
    }
}
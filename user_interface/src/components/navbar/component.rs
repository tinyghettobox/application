use std::sync::{Arc};
use std::sync::Mutex;
use gtk4::prelude::IsA;
use gtk4::Widget;
use tracing::debug;
use crate::components::{Children, Component};
use crate::components::navbar::widget::NavbarWidget;
use crate::state::{Action, Dispatcher, Event, EventHandler, State};

pub struct NavbarComponent {
    pub widget: NavbarWidget,
    pub children: Vec<Arc<Mutex<Box<dyn EventHandler>>>>,
    state: Arc<Mutex<State>>,
}

impl EventHandler for NavbarComponent {
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

impl Component<Option<()>> for NavbarComponent {
    fn new(state: Arc<Mutex<State>>, dispatcher: Arc<Mutex<Dispatcher>>, params: Option<()>) -> Self {
        let (widget, children) = Self::render(state.clone(), dispatcher.clone(), params);
        let mut component = NavbarComponent { state, widget, children };
        component.update();
        component
    }

    fn render(state: Arc<Mutex<State>>, dispatcher: Arc<Mutex<Dispatcher>>, _params: Option<()>) -> (NavbarWidget, Children) {
        let navbar = NavbarWidget::new();
        navbar.connect_back_clicked(move |_| {
            debug!("connect_back_clicked start");
            let parent_id = state.lock().expect("could not lock state").library_entry.parent_id;

            dispatcher
                .lock()
                .expect("Could not lock dispatcher")
                .dispatch_action(Action::Select(parent_id.unwrap()));

            debug!("connect_back_clicked end");
        });

        (navbar, vec![])
    }

    fn update(&mut self) {
        debug!("navbar update start");
        let state = self.state.lock().expect("Could not lock state");
        if state.library_entry.id == 0 {
            self.widget.set_visibility(false);
        } else {
            self.widget.set_visibility(true);
            self.widget.set_image(state.library_entry.image.clone());
            self.widget.set_name(state.library_entry.name.clone());
        }
        debug!("navbar update end");
    }

    fn get_widget(&self) -> impl IsA<Widget> {
        self.widget.clone()
    }
}

use crate::components::navbar::widget::NavbarWidget;
use crate::components::{Children, Component};
use crate::state::{Action, Dispatcher, Event, EventHandler, State};
use std::sync::{Arc, Mutex};

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
    fn new(
        state: Arc<Mutex<State>>,
        dispatcher: Arc<Mutex<Dispatcher>>,
        params: Option<()>,
    ) -> Self {
        let (widget, children) = Self::render(state.clone(), dispatcher.clone(), params);
        let mut component = NavbarComponent {
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
    ) -> (NavbarWidget, Children) {
        let navbar = NavbarWidget::new();

        {
            let dispatcher = dispatcher.clone();
            navbar.connect_back_clicked(move |_| {
                let parent_id = state.lock().unwrap().library_entry.parent_id;
                dispatcher
                    .lock()
                    .unwrap()
                    .dispatch_action(Action::Select(parent_id.unwrap()));
            });
        }
        {
            let dispatcher = dispatcher.clone();
            navbar.connect_show_log_clicked(move |_| {
                dispatcher
                    .lock()
                    .unwrap()
                    .dispatch_action(Action::ToggleLogOverlay(true));
            })
        }

        (navbar, vec![])
    }

    fn update(&mut self) {
        let state = self.state.lock().unwrap();
        if state.library_entry.id == 0 {
            self.widget.set_visibility(false);
        } else {
            self.widget.set_visibility(true);
            self.widget.set_image(state.library_entry.image.clone());
            self.widget.set_name(state.library_entry.name.clone());
        }
    }
    #[allow(refining_impl_trait)]
    fn get_widget(&self) -> NavbarWidget {
        self.widget.clone()
    }
}

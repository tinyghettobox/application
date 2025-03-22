use std::sync::{Arc, Mutex};

use gtk4::prelude::GtkWindowExt;
use gtk4::Application;

use crate::components::content::ContentComponent;
use crate::components::log_overlay::LogOverlayComponent;
use crate::components::navbar::NavbarComponent;
use crate::components::player_bar::PlayerBarComponent;
use crate::components::ripple::RippleComponent;
use crate::components::shutdown_timer::ShutdownTimerComponent;
use crate::components::window::widget::WindowWidget;
use crate::components::{Children, Component};
use crate::state::{Dispatcher, Event, EventHandler, State};

pub struct WindowComponent {
    pub widget: WindowWidget,
    pub children: Vec<Arc<Mutex<Box<dyn EventHandler>>>>,
}

impl EventHandler for WindowComponent {
    fn on_event(&mut self, event: &Event) {
        match event {
            Event::Error(error) => tracing::error!("Received error event: {}", error),
            _ => {}
        }
    }

    fn get_children(&self) -> Vec<Arc<Mutex<Box<dyn EventHandler>>>> {
        self.children.clone()
    }
}

impl Component<Option<()>> for WindowComponent {
    fn new(
        state: Arc<Mutex<State>>,
        dispatcher: Arc<Mutex<Dispatcher>>,
        params: Option<()>,
    ) -> Self {
        let (widget, children) = Self::render(state.clone(), dispatcher.clone(), params);
        let mut component = Self { widget, children };
        component.update();
        component
    }

    #[allow(refining_impl_trait)]
    fn render(
        state: Arc<Mutex<State>>,
        dispatcher: Arc<Mutex<Dispatcher>>,
        _params: Option<()>,
    ) -> (WindowWidget, Children) {
        let shutdown_timer = ShutdownTimerComponent::new(state.clone(), dispatcher.clone(), None);
        let log_overlay = LogOverlayComponent::new(state.clone(), dispatcher.clone(), None);
        let ripple = RippleComponent::new(state.clone(), dispatcher.clone(), None);
        let navbar = NavbarComponent::new(state.clone(), dispatcher.clone(), None);
        let content = ContentComponent::new(state.clone(), dispatcher.clone(), None);
        let player_bar = PlayerBarComponent::new(state.clone(), dispatcher.clone(), None);

        log_overlay.add_child(&navbar.get_widget());
        log_overlay.add_child(&content.get_widget());
        log_overlay.add_child(&player_bar.get_widget());

        ripple.add_child(&log_overlay.get_widget());

        shutdown_timer.add_child(&ripple.get_widget());

        let widget = WindowWidget::new();
        widget.connect_close_request(|_| {
            std::process::exit(0);
        });

        widget.set_child(Some(&shutdown_timer.get_widget()));

        (
            widget,
            vec![
                Arc::new(Mutex::new(Box::new(navbar))),
                Arc::new(Mutex::new(Box::new(content))),
                Arc::new(Mutex::new(Box::new(player_bar))),
                Arc::new(Mutex::new(Box::new(ripple))),
                Arc::new(Mutex::new(Box::new(log_overlay))),
            ],
        )
    }

    fn update(&mut self) {}

    #[allow(refining_impl_trait)]
    fn get_widget(&self) -> WindowWidget {
        self.widget.clone()
    }
}

impl WindowComponent {
    pub fn present(&self, app: &Application) {
        self.widget.set_application(Some(app));
        self.widget.present();
    }
}

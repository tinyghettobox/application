use crate::components::log_overlay::widget::LogOverlayWidget;
use crate::components::{Children, Component};
use crate::state::{Action, Dispatcher, Event, EventHandler, State};
use gtk4::gio::ListStore;
use gtk4::glib::BoxedAnyObject;
use gtk4::prelude::{IsA, ListModelExtManual};
use gtk4::Widget;
use std::sync::{Arc, Mutex};
use tracing::info;

pub struct LogOverlayComponent {
    widget: LogOverlayWidget,
    state: Arc<Mutex<State>>,
}

impl EventHandler for LogOverlayComponent {
    fn on_event(&mut self, event: &Event) {
        match event {
            Event::LogOverlayToggled | Event::LogLevelChanged => self.update(),
            _ => {}
        }
    }

    fn get_children(&self) -> Vec<Arc<Mutex<Box<dyn EventHandler>>>> {
        vec![]
    }
}

impl Component<Option<()>> for LogOverlayComponent {
    fn new(
        state: Arc<Mutex<State>>,
        dispatcher: Arc<Mutex<Dispatcher>>,
        params: Option<()>,
    ) -> Self {
        let (widget, _) = Self::render(state.clone(), dispatcher.clone(), params);
        let mut component = Self { widget, state };
        component.update();
        component
    }

    #[allow(refining_impl_trait)]
    fn render(
        _state: Arc<Mutex<State>>,
        dispatcher: Arc<Mutex<Dispatcher>>,
        _params: Option<()>,
    ) -> (LogOverlayWidget, Children) {
        let widget = LogOverlayWidget::new();
        widget.set_factory();
        {
            let dispatcher = dispatcher.clone();
            widget.connect_close_clicked(move |_| {
                dispatcher
                    .lock()
                    .expect("could not lock")
                    .dispatch_action(Action::ToggleLogOverlay(false));
            });
        }
        {
            let dispatcher = dispatcher.clone();
            widget.connect_log_level_changed(move |level| {
                dispatcher
                    .lock()
                    .expect("could not lock")
                    .dispatch_action(Action::SetLogLevel(level));
            });
        }

        (widget, vec![])
    }

    fn update(&mut self) {
        let show_log_overlay = self.state.lock().expect("could not lock").show_log_overlay;
        let log_level = self
            .state
            .lock()
            .expect("could not lock")
            .log_level
            .to_string();

        if show_log_overlay {
            let messages = self
                .state
                .lock()
                .expect("could not lock")
                .messages
                .lock()
                .expect("could not lock messages")
                .iter()
                .filter(|message| level_to_number(&message.level) >= level_to_number(&log_level))
                .map(|message| BoxedAnyObject::new(message.clone()))
                .collect::<ListStore>();

            info!(
                "Showing {} log messages",
                messages.iter::<BoxedAnyObject>().len()
            );

            self.widget.set_list_store(messages);
            self.widget.set_visible(true);
        } else {
            self.widget
                .set_list_store(ListStore::new::<BoxedAnyObject>());
            self.widget.set_visible(false);
        }
    }

    fn get_widget(&self) -> impl IsA<Widget> {
        self.widget.clone()
    }
}

impl LogOverlayComponent {
    pub fn add_child(&self, widget: &impl IsA<Widget>) {
        self.widget.add_child(widget);
    }
}

fn level_to_number(level: &str) -> u8 {
    match level {
        "TRACE" => 0,
        "DEBUG" => 1,
        "INFO" => 2,
        "WARN" => 3,
        "ERROR" => 4,
        _ => panic!("Unknown log level {}", level), // unknown
    }
}

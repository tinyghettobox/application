use super::widget::NotificationWidget;
use crate::components::{Children, Component};
use crate::state::{Dispatcher, Event, EventHandler, State};
use gtk4::prelude::IsA;
use gtk4::Widget;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tracing::info;

#[derive(Clone)]
pub struct NotificationComponent {
    widget: NotificationWidget,
    messages: Arc<Mutex<VecDeque<String>>>,
}

impl EventHandler for NotificationComponent {
    fn on_event(&mut self, event: &Event) {
        match event {
            Event::Error(msg) => self.add_message("error", msg),
            _ => {}
        }
    }

    fn get_children(&self) -> Vec<Arc<Mutex<Box<dyn EventHandler>>>> {
        vec![]
    }
}

impl Component<Option<()>> for NotificationComponent {
    fn new(
        state: Arc<Mutex<State>>,
        dispatcher: Arc<Mutex<Dispatcher>>,
        params: Option<()>,
    ) -> Self {
        let (widget, _) = Self::render(state.clone(), dispatcher.clone(), params);
        let mut component = Self {
            widget,
            messages: Arc::new(Mutex::new(VecDeque::new())),
        };
        component.update();
        component.start_listener();
        component
    }

    #[allow(refining_impl_trait)]
    fn render(
        _state: Arc<Mutex<State>>,
        _dispatcher: Arc<Mutex<Dispatcher>>,
        _params: Option<()>,
    ) -> (NotificationWidget, Children) {
        let widget = NotificationWidget::new();
        (widget, vec![])
    }

    fn update(&mut self) {
        // Only show next message if not currently revealed
        if self.widget.is_revealed() {
            return;
        }
        let msgs = self.messages.lock().unwrap();
        if let Some(msg) = msgs.front() {
            self.widget.show_notification(msg);
        }
    }

    fn get_widget(&self) -> impl IsA<Widget> {
        self.widget.clone()
    }
}

impl NotificationComponent {
    pub fn add_child(&self, widget: &impl IsA<Widget>) {
        self.widget.add_child(widget);
    }

    fn start_listener(&self) {
        let self_ = self.clone();
        self_.widget.clone().connect_close_clicked(move |_| {
            self_.widget.hide_notification();
            self_.messages.lock().unwrap().pop_front();
            {
                let mut self_ = self_.clone();
                self_.update();
            }
        });
    }

    fn add_message(&mut self, level: &str, message: &str) {
        self.messages.lock().unwrap().push_back(message.to_string());
        info!("New notification: [{}] {}", level, message);
        self.update();
    }
}

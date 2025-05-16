use super::widget::ShutdownTimerWidget;
use crate::components::{Children, Component};
use crate::state::{Action, Dispatcher, Event, EventHandler, State};
use gtk4::prelude::IsA;
use gtk4::{glib, Widget};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::info;

pub struct ShutdownTimerComponent {
    children: Vec<Arc<Mutex<Box<dyn EventHandler>>>>,
    widget: ShutdownTimerWidget,
    state: Arc<Mutex<State>>,
}

impl EventHandler for ShutdownTimerComponent {
    fn on_event(&mut self, event: &Event) {
        match event {
            Event::MonitorToggled => self.update(),
            _ => {}
        }
    }

    fn get_children(&self) -> Vec<Arc<Mutex<Box<dyn EventHandler>>>> {
        self.children.clone()
    }
}

impl Component<Option<()>> for ShutdownTimerComponent {
    fn new(
        state: Arc<Mutex<State>>,
        dispatcher: Arc<Mutex<Dispatcher>>,
        params: Option<()>,
    ) -> Self {
        let (widget, children) = Self::render(state.clone(), dispatcher, params);
        let mut component = Self {
            children,
            widget,
            state,
        };
        component.update();
        component
    }

    #[allow(refining_impl_trait)]
    fn render(
        state: Arc<Mutex<State>>,
        dispatcher: Arc<Mutex<Dispatcher>>,
        _params: Option<()>,
    ) -> (ShutdownTimerWidget, Children) {
        let widget = ShutdownTimerWidget::new();

        {
            let state = state.clone();
            let dispatcher = dispatcher.clone();
            widget.connect_clicked(move || {
                let monitor_active = state.lock().expect("could not lock").monitor_active;
                if !monitor_active {
                    dispatcher
                        .lock()
                        .expect("could not lock")
                        .dispatch_action(Action::ToggleMonitor(true));
                }
                dispatcher
                    .lock()
                    .expect("could not lock")
                    .dispatch_action(Action::CaptureActivity);
            });
        }

        let state = state.clone();
        glib::timeout_add_local(Duration::from_secs(1), move || {
            let state = state.lock().expect("could not lock");
            let last_activity = state.last_activity;
            let monitor_active = state.monitor_active;
            let display_off_timeout = Duration::from_secs(state.display_off_timeout as u64 * 60);
            let sleep_timeout = Duration::from_secs(state.sleep_timeout as u64 * 60);

            if last_activity.elapsed() > display_off_timeout && monitor_active {
                dispatcher
                    .lock()
                    .expect("could not lock")
                    .dispatch_action(Action::ToggleMonitor(false));
            }
            if last_activity.elapsed() > sleep_timeout {
                dispatcher
                    .lock()
                    .expect("could not lock")
                    .dispatch_action(Action::Shutdown);
            }

            glib::ControlFlow::Continue
        });

        (widget, vec![])
    }

    fn update(&mut self) {
        let monitor_active = self.state.lock().expect("could not lock").monitor_active;
        info!("Setting forward to {}", monitor_active);
        self.widget.set_forwards_clicks(monitor_active);
    }

    fn get_widget(&self) -> impl IsA<Widget> {
        self.widget.clone()
    }
}

impl ShutdownTimerComponent {
    pub fn add_child(&self, widget: &impl IsA<Widget>) {
        self.widget.add_child(widget);
    }
}

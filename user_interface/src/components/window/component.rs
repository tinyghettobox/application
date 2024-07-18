use std::sync::{Arc};
use std::sync::Mutex;
use gtk4::prelude::{GtkWindowExt, IsA};
use gtk4::{Application, Widget};
use crate::components::{Children, Component};
use crate::components::content::ContentComponent;
use crate::components::navbar::NavbarComponent;
use crate::components::player_bar::PlayerBarComponent;
use crate::components::window::widget::WindowWidget;
use crate::state::{Dispatcher, Event, EventHandler, State};


pub struct WindowComponent {
    pub widget: WindowWidget,
    pub children: Vec<Arc<Mutex<Box<dyn EventHandler>>>>,
}

impl EventHandler for WindowComponent {
    fn on_event(&mut self, _event: &Event) {

    }

    fn get_children(&self) -> Vec<Arc<Mutex<Box<dyn EventHandler>>>> {
        self.children.clone()
    }
}

impl Component<Option<()>> for WindowComponent {
    fn new(state: Arc<Mutex<State>>, dispatcher: Arc<Mutex<Dispatcher>>, params: Option<()>) -> Self {
        let (widget, children) = Self::render(state.clone(), dispatcher.clone(), params);
        let mut component = Self { widget, children };
        component.update();
        component
    }

    fn render(state: Arc<Mutex<State>>, dispatcher: Arc<Mutex<Dispatcher>>, _params: Option<()>) -> (WindowWidget, Children) {
        let navbar = NavbarComponent::new(state.clone(), dispatcher.clone(), None);
        let content = ContentComponent::new(state.clone(), dispatcher.clone(), None);
        let player_bar = PlayerBarComponent::new(state.clone(), dispatcher.clone(), None);

        let widget = WindowWidget::new();
        widget.add_child(&navbar.widget);
        widget.add_child(&content.widget);
        widget.add_child(&player_bar.widget);

        (widget, vec![Arc::new(Mutex::new(Box::new(navbar))), Arc::new(Mutex::new(Box::new(content))), Arc::new(Mutex::new(Box::new(player_bar)))])
    }

    fn update(&mut self) {

    }

    fn get_widget(&self) -> impl IsA<Widget> {
        self.widget.clone()
    }
}

impl WindowComponent {
    pub fn present(&self, app: &Application) {
        self.widget.set_application(Some(app));
        self.widget.present();
    }
}
use std::sync::{Arc};
use std::sync::Mutex;
use gtk4::prelude::IsA;
use gtk4::Widget;
use tracing::debug;
use crate::components::{Children, Component};
use crate::components::player_bar::widget::PlayerBarWidget;
use crate::state::{Action, Dispatcher, Event, EventHandler, State};

pub struct PlayerBarComponent {
    pub(crate) widget: PlayerBarWidget,
    pub(crate) children: Vec<Arc<Mutex<Box<dyn EventHandler>>>>,
    state: Arc<Mutex<State>>,
}

impl EventHandler for PlayerBarComponent {
    fn on_event(&mut self, event: &Event) {
        match event {
            Event::TrackChanged => {
                self.update_play_state();
                self.update_track();
            },
            // Event::PlayStateChanged => self.update_play_state(),
            // Event::ProgressChanged => self.update_progress(),
            _ => {}
        }
    }

    fn get_children(&self) -> Vec<Arc<Mutex<Box<dyn EventHandler>>>> {
        self.children.clone()
    }
}

impl Component<Option<()>> for PlayerBarComponent {
    fn new(state: Arc<Mutex<State>>, dispatcher: Arc<Mutex<Dispatcher>>, params: Option<()>) -> Self {
        let (widget, children) = Self::render(state.clone(), dispatcher.clone(), params);
        let mut component = Self { widget, children, state };
        component.update();
        component
    }

    fn render(_state: Arc<Mutex<State>>, dispatcher: Arc<Mutex<Dispatcher>>, _params: Option<()>) -> (PlayerBarWidget, Children) {
        let widget = PlayerBarWidget::new();

        {
            let dispatcher = dispatcher.clone();
            widget.connect_play_toggle_clicked(move || {
                dispatcher.lock().expect("Could not lock dispatcher").dispatch_action(Action::TogglePlay);
            });
        }
        {
            let dispatcher = dispatcher.clone();
            widget.connect_back_clicked(move || {
                dispatcher.lock().expect("Could not lock dispatcher").dispatch_action(Action::PrevTrack);
            });
        }
        {
            let dispatcher = dispatcher.clone();
            widget.connect_forward_clicked(move || {
                dispatcher.lock().expect("Could not lock dispatcher").dispatch_action(Action::NextTrack);
            });
        }
        {
            let dispatcher = dispatcher.clone();
            widget.connect_seek(move |progress| {
                debug!("connect_seek start");
                dispatcher.lock().expect("Could not lock dispatcher").dispatch_action(Action::Seek(progress));
                debug!("connect_seek end");
            });
        }

        (widget, vec![])
    }

    fn update(&mut self) {
        self.update_progress();
        self.update_track();
        self.update_play_state();
    }

    fn get_widget(&self) -> impl IsA<Widget> {
        self.widget.clone()
    }
}

impl PlayerBarComponent {
    pub fn update_progress(&self) {
        debug!("updating progress");
        let state = self.state.lock().expect("Could not lock state");
        debug!("updating progress locked");
        self.widget.set_progress(state.progress);
        debug!("updated progress");
    }

    pub fn update_track(&self) {
        debug!("player_bar update_track start");
        let state = self.state.lock().expect("Could not lock state");
        if let Some(playing_library_entry) = state.playing_library_entry.as_ref() {
            self.widget.set_visibility(true);
            self.widget.set_image(playing_library_entry.image.clone());
            self.widget.set_track_name(playing_library_entry.name.clone());
            self.widget.set_folder_name(playing_library_entry.parent_name.clone().unwrap_or("".to_string()));
        } else {
            self.widget.set_visibility(false);
        }
        debug!("player_bar update_track end");
    }

    pub fn update_play_state(&self) {
        debug!("player_bar update_play_state start");
        let state = self.state.lock().expect("Could not lock state");
        self.widget.set_paused(state.paused);
        debug!("player_bar update_play_state end");
    }
}
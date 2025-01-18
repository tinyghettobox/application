use crate::components::player_bar::widget::PlayerBarWidget;
use crate::components::{Children, Component};
use crate::state::{Action, Dispatcher, Event, EventHandler, State};
use crate::util::debouncer::Debouncer;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::warn;

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
            }
            Event::PlayStateChanged => self.update_play_state(),
            Event::ProgressChanged => self.update_progress(),
            Event::VolumeChanged => self.update_volume(),
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
        let mut component = Self {
            widget,
            children,
            state,
        };
        component.update();
        component
    }

    #[allow(refining_impl_trait)]
    fn render(
        _state: Arc<Mutex<State>>,
        dispatcher: Arc<Mutex<Dispatcher>>,
        _params: Option<()>,
    ) -> (PlayerBarWidget, Children) {
        let widget = PlayerBarWidget::new();

        {
            let dispatcher = dispatcher.clone();
            widget.connect_play_toggle_clicked(move || {
                dispatcher.lock().unwrap().dispatch_action(Action::TogglePlay);
            });
        }
        {
            let dispatcher = dispatcher.clone();
            widget.connect_back_clicked(move || {
                dispatcher.lock().unwrap().dispatch_action(Action::PrevTrack);
            });
        }
        {
            let dispatcher = dispatcher.clone();
            widget.connect_forward_clicked(move || {
                dispatcher.lock().unwrap().dispatch_action(Action::NextTrack);
            });
        }
        {
            let dispatcher = dispatcher.clone();
            let debouncer = Debouncer::new(Duration::from_millis(500), move |progress| {
                dispatcher.lock().unwrap().dispatch_action(Action::Seek(progress));
            });
            widget.connect_seek(move |progress| debouncer.add(progress));
        }
        {
            let dispatcher = dispatcher.clone();
            let debouncer = Debouncer::new(Duration::from_millis(500), move |volume| {
                dispatcher.lock().unwrap().dispatch_action(Action::SetVolume(volume));
            });
            widget.connect_volume_change(move |volume: f64| debouncer.add(volume));
        }

        (widget, vec![])
    }

    fn update(&mut self) {
        self.update_progress();
        self.update_track();
        self.update_play_state();
        self.update_volume();
    }

    #[allow(refining_impl_trait)]
    fn get_widget(&self) -> PlayerBarWidget {
        self.widget.clone()
    }
}

impl PlayerBarComponent {
    pub fn update_progress(&self) {
        let state = self.state.lock().unwrap();
        self.widget.set_progress(state.progress);
    }

    pub fn update_track(&self) {
        let state = self.state.lock().unwrap();
        if let Some(playing_library_entry) = state.playing_library_entry.as_ref() {
            self.widget.set_visibility(true);
            self.widget.set_image(playing_library_entry.image.clone().or(playing_library_entry.parent_image.clone()));
            self.widget.set_track_name(playing_library_entry.name.clone());
            self.widget.set_folder_name(playing_library_entry.parent_name.clone().unwrap_or("".to_string()));
        } else {
            self.widget.set_visibility(false);
        }
    }

    pub fn update_play_state(&self) {
        let state = self.state.lock().unwrap();
        self.widget.set_paused(state.paused);
    }

    pub fn update_volume(&self) {
        let state = self.state.lock().unwrap();
        self.widget.set_volume(state.volume);
    }
}

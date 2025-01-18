use gtk4::glib::clone;
use gtk4::glib::property::PropertyGet;
use gtk4::prelude::{BoxExt, ObjectExt};
use std::sync::{Arc, Mutex};
use tracing::{error, warn};

use crate::components::detail_list_item::widget::{DetailListItemState, DetailListItemWidget};
use crate::components::{Children, Component};
use crate::state::{Action, Dispatcher, Event, EventHandler, State};
use database::model::library_entry::Model as LibraryEntry;

#[derive(Clone)]
pub struct DetailListItemComponent {
    pub widget: Arc<DetailListItemWidget>,
    pub children: Vec<Arc<Mutex<Box<dyn EventHandler>>>>,
    state: Arc<Mutex<State>>,
    library_entry: Arc<Mutex<Option<LibraryEntry>>>,
}

impl EventHandler for DetailListItemComponent {
    fn on_event(&mut self, event: &Event) {
        match event {
            Event::LibraryEntryChanged | Event::PlayStateChanged | Event::TrackChanged | Event::TrackPlayed => {
                self.update();
            }
            _ => {}
        }
    }

    fn get_children(&self) -> Vec<Arc<Mutex<Box<dyn EventHandler>>>> {
        self.children.clone()
    }
}

impl Component<Option<()>> for DetailListItemComponent {
    fn new(state: Arc<Mutex<State>>, dispatcher: Arc<Mutex<Dispatcher>>, params: Option<()>) -> Self {
        let (widget, children) = Self::render(state.clone(), dispatcher.clone(), params);

        let mut component = Self {
            widget: Arc::new(widget.clone()),
            children,
            state,
            library_entry: Default::default(),
        };

        widget.set_component(Box::new(component.clone()));
        widget.connect_clicked(clone!(
            #[strong]
            component,
            move || {
                let library_entry =
                    component.library_entry.lock().expect("could not lock").clone().expect("No library entry set");
                let dispatcher = dispatcher.clone();
                dispatcher.lock().unwrap().dispatch_action(Action::Play(
                    library_entry.parent_id.expect("A children should have a parent"),
                    Some(library_entry.id),
                ));
            }
        ));

        component
    }

    #[allow(refining_impl_trait)]
    fn render(
        _state: Arc<Mutex<State>>,
        _dispatcher: Arc<Mutex<Dispatcher>>,
        _params: Option<()>,
    ) -> (DetailListItemWidget, Children) {
        let widget = DetailListItemWidget::new();

        (widget, vec![])
    }

    fn update(&mut self) {
        let library_entry_id = match self.library_entry.lock().expect("could not lock").as_ref() {
            Some(entry) => entry.id,
            None => {
                warn!("No library entry set for detail list item");
                return;
            }
        };
        let playing_library_entry_id = self.state.lock().unwrap().playing_library_entry.clone().map(|entry| entry.id);
        let entry_with_position = self.state.lock().unwrap().library_entry.children.clone().and_then(|children| {
            let position = children.iter().position(|child| child.id == library_entry_id);
            position.and_then(move |pos| children.get(pos).cloned().map(|entry| (pos, entry)))
        });

        match entry_with_position {
            Some((position, entry)) => {
                self.widget.set_position(position as u32);
                self.widget.set_name(&entry.name);
                tracing::debug!("entry_id: {}, playing id: {:?}", entry.id, playing_library_entry_id);
                // Is this currently playing?
                if entry.id == playing_library_entry_id.unwrap_or(-1) {
                    self.widget.set_state(DetailListItemState::Playing);
                } else if let Some(_) = entry.played_at.as_ref() {
                    self.widget.set_state(DetailListItemState::Played);
                } else {
                    self.widget.set_state(DetailListItemState::None);
                }
            }
            None => {
                error!("Wanted to render detail list item but not having children? o.O");
            }
        }
    }

    #[allow(refining_impl_trait)]
    fn get_widget(&self) -> DetailListItemWidget {
        (*self.widget).clone()
    }
}

/**
 * Allows passing component as listener for the library entry to the widget.
 * This construct is necessary due to the way the list component works. A SignalListItemFactory is
 * binding the library entry on demand and this need to be forwarded to the component.
 */
pub trait ListenerComponent {
    fn notify(&mut self, value: Option<LibraryEntry>);
}

impl ListenerComponent for DetailListItemComponent {
    fn notify(&mut self, value: Option<LibraryEntry>) {
        *self.library_entry.lock().expect("could not lock") = value;
        self.update();
    }
}

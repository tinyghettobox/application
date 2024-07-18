use database::model::library_entry::Model as LibraryEntry;
use std::sync::{Arc};
use std::sync::Mutex;
use gtk4::{CompositeTemplate, glib, ListItem, NoSelection, SignalListItemFactory};
use gtk4::gio::ListStore;
use gtk4::glib::{BoxedAnyObject, object_subclass};
use gtk4::glib::subclass::InitializingObject;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use tracing::debug;
use crate::components::Component;
use crate::components::detail_list_item::{DetailListItemComponent, DetailListItemState, DetailListItemWidget};
use crate::state::{Dispatcher, State};

#[derive(Default, CompositeTemplate)]
#[template(file = "./detail_list.ui")]
pub struct DetailListWidgetImp {
    #[template_child]
    list: TemplateChild<gtk4::ListView>
}

#[object_subclass]
impl ObjectSubclass for DetailListWidgetImp {
    const NAME: &'static str = "MupiboxDetailList";
    type Type = DetailListWidget;
    type ParentType = gtk4::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for DetailListWidgetImp {}
impl WidgetImpl for DetailListWidgetImp {}
impl BoxImpl for DetailListWidgetImp {}

glib::wrapper! {
    pub struct DetailListWidget(ObjectSubclass<DetailListWidgetImp>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl DetailListWidget {
    pub fn new(state: Arc<Mutex<State>>, dispatcher: Arc<Mutex<Dispatcher>>) -> Self {
        let widget: Self = glib::Object::new();
        widget.set_factory(state, dispatcher);
        widget
    }

    fn set_factory(&self, state: Arc<Mutex<State>>, dispatcher: Arc<Mutex<Dispatcher>>) {
        let state_ = state.clone();
        let factory = SignalListItemFactory::new();
        factory.connect_setup(move |_, list_item| {
            let component = DetailListItemComponent::new(state.clone(), dispatcher.clone(), None);
            let item = list_item
                .downcast_ref::<ListItem>()
                .expect("Needs to be ListItem");

            item.set_child(Some(&component.widget));
        });

        factory.connect_bind(move |_, list_item| {
            debug!("update factory bind");
            let list_item = list_item
                .downcast_ref::<ListItem>()
                .expect("could not downcast to list item");
            let boxed = list_item
                .item()
                .and_downcast::<BoxedAnyObject>()
                .expect("Could not downcast to boxed any object");
            let library_entry = boxed.borrow::<LibraryEntry>();

            let widget = list_item
                .child()
                .and_downcast::<DetailListItemWidget>()
                .expect("could not downcast to widget");

            widget.set_position(list_item.position());
            widget.set_library_entry(library_entry.clone());
            widget.set_name(&library_entry.name);

            let state = state_.lock().expect("could not lock state");
            if library_entry.id == state.playing_library_entry.clone().map(|entry| entry.id).unwrap_or(-1) {
                widget.set_state(DetailListItemState::Playing);
            }
            else if library_entry.played_at.is_some() {
                widget.set_state(DetailListItemState::Playing);
            }
            else {
                widget.set_state(DetailListItemState::None);
            }
            debug!("update factory bind done");
        });

        factory.connect_unbind(|_, list_item| {
            println!("Unbind");
            list_item
                .downcast_ref::<ListItem>()
                .expect("Needs to be ListItem")
                .set_child(None::<&DetailListItemWidget>);
        });

        self.imp().list.set_factory(Some(&factory));
    }

    pub fn set_list_store(&self, store: ListStore) {
        self.imp().list.set_model(Some(&NoSelection::new(Some(store))));
    }
}

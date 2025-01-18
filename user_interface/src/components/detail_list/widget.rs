use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::components::detail_list_item::{DetailListItemComponent, DetailListItemWidget};
use crate::components::Component;
use crate::state::{Dispatcher, EventHandler, State};
use database::model::library_entry::Model as LibraryEntry;
use gtk4::gio::ListStore;
use gtk4::glib::subclass::InitializingObject;
use gtk4::glib::{object_subclass, BoxedAnyObject};
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::{glib, CompositeTemplate, ListItem, NoSelection, SignalListItemFactory};

#[derive(Default, CompositeTemplate)]
#[template(file = "./detail_list.ui")]
pub struct DetailListWidgetImp {
    #[template_child]
    list: TemplateChild<gtk4::ListView>,
}

#[object_subclass]
impl ObjectSubclass for DetailListWidgetImp {
    const NAME: &'static str = "TinyGhettoBoxDetailList";
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
    pub fn new() -> Self {
        let widget: Self = glib::Object::new();
        widget
    }

    pub fn set_factory(
        &self,
        state: Arc<Mutex<State>>,
        dispatcher: Arc<Mutex<Dispatcher>>,
        children: Rc<RefCell<Vec<Arc<Mutex<Box<dyn EventHandler>>>>>>,
    ) {
        let factory = SignalListItemFactory::new();

        factory.connect_setup(move |_, list_item| {
            let state = state.clone();
            let dispatcher = dispatcher.clone();
            let component = DetailListItemComponent::new(state, dispatcher, None);

            let item = list_item.downcast_ref::<ListItem>().expect("Needs to be ListItem");

            item.set_child(Some(&component.get_widget()));

            let mut children = children.borrow_mut();
            children.push(Arc::new(Mutex::new(Box::new(component))));
        });

        factory.connect_bind(move |_, list_item| {
            let list_item = list_item.downcast_ref::<ListItem>().expect("could not downcast to list item");
            let boxed =
                list_item.item().and_downcast::<BoxedAnyObject>().expect("Could not downcast to boxed any object");
            let library_entry = boxed.borrow::<LibraryEntry>();

            let widget =
                list_item.child().and_downcast::<DetailListItemWidget>().expect("could not downcast to widget");

            widget.set_library_entry(library_entry.clone());
        });

        factory.connect_unbind(|_, list_item| {
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

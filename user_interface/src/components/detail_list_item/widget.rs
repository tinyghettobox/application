use std::cell::RefCell;
use std::sync::{Arc, Mutex};

use super::component::ListenerComponent;
use crate::state::{Action, Dispatcher};
use database::model::library_entry::Model as LibraryEntry;
use gtk4::glib::object_subclass;
use gtk4::glib::subclass::InitializingObject;
use gtk4::prelude::{GestureExt, ObjectExt, WidgetExt};
use gtk4::subclass::prelude::*;
use gtk4::{glib, CompositeTemplate, GestureClick};
use tracing::info;

#[derive(Default, CompositeTemplate)]
#[template(file = "./detail_list_item.ui")]
pub struct DetailListItemWidgetImp {
    #[template_child]
    position: TemplateChild<gtk4::Label>,
    #[template_child]
    name: TemplateChild<gtk4::Label>,
    #[template_child]
    icon: TemplateChild<gtk4::Image>,
    #[template_child]
    wrapper: TemplateChild<gtk4::Box>,

    component: RefCell<Option<Box<dyn ListenerComponent>>>,
}

#[object_subclass]
impl ObjectSubclass for DetailListItemWidgetImp {
    const NAME: &'static str = "TinyGhettoBoxDetailListItem";
    type Type = DetailListItemWidget;
    type ParentType = gtk4::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}
impl ObjectImpl for DetailListItemWidgetImp {}
impl WidgetImpl for DetailListItemWidgetImp {}
impl BoxImpl for DetailListItemWidgetImp {}

glib::wrapper! {
    pub struct DetailListItemWidget(ObjectSubclass<DetailListItemWidgetImp>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl DetailListItemWidget {
    pub fn new() -> Self {
        glib::Object::new()
    }

    /**
     * Allows passing component as listener for the library entry. This construct is necessary due to
     * the way the list component works. A SignalListItemFactory is binding the library entry on
     * demand and this need to be forwarded to the component.
     */
    pub fn set_component(&self, component: Box<dyn ListenerComponent>) {
        *self.imp().component.borrow_mut() = Some(component);
    }

    /**
     * Forward the library entry from SignalListItemFactory to the component.
     */
    pub fn set_library_entry(&self, library_entry: LibraryEntry) {
        self.imp().component.borrow_mut().as_mut().map(|component| component.notify(Some(library_entry)));
    }

    pub fn set_position(&self, position: u32) {
        self.imp().position.set_label(format!("#{}", position).as_str());
    }

    pub fn set_name(&self, name: &str) {
        self.imp().name.set_label(name);
    }

    pub fn set_state(&self, state: DetailListItemState) {
        match state {
            DetailListItemState::Playing => {
                self.imp().icon.set_icon_name(Some("play"));
                self.imp().icon.set_css_classes(&vec!["status-icon", "playing"]);
            }
            DetailListItemState::Played => {
                self.imp().icon.set_icon_name(Some("check"));
                self.imp().icon.set_css_classes(&vec!["status-icon", "played"]);
            }
            DetailListItemState::None => {
                self.imp().icon.set_icon_name(None);
                self.imp().icon.set_css_classes(&vec!["status-icon"]);
            }
        }
    }

    pub fn connect_clicked(&self, callback: impl Fn() + Send + Sync + 'static) {
        let gesture = GestureClick::new();
        gesture.connect_released(move |gesture, _, _, _| {
            gesture.set_state(gtk4::EventSequenceState::Claimed);
            callback();
        });
        self.imp().wrapper.add_controller(gesture);
    }
}

#[derive(Debug)]
pub enum DetailListItemState {
    Playing,
    Played,
    None,
}

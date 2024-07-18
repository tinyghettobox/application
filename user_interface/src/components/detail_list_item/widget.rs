use std::cell::RefCell;
use std::sync::{Arc};
use std::sync::Mutex;
use gtk4::{CompositeTemplate, GestureClick, glib};
use gtk4::glib::object_subclass;
use gtk4::glib::subclass::InitializingObject;
use gtk4::prelude::{GestureExt, WidgetExt};
use gtk4::subclass::prelude::*;
use tracing::debug;
use crate::state::{Action, Dispatcher};
use database::model::library_entry::Model as LibraryEntry;

#[derive(Default, CompositeTemplate)]
#[template(file = "./detail_list_item.ui")]
pub struct DetailListItemWidgetImp {
    #[template_child]
    position: TemplateChild<gtk4::Label>,
    #[template_child]
    name: TemplateChild<gtk4::Label>,
    #[template_child]
    icon: TemplateChild<gtk4::Image>,

    library_entry: RefCell<Option<LibraryEntry>>,
}

#[object_subclass]
impl ObjectSubclass for DetailListItemWidgetImp {
    const NAME: &'static str = "MupiboxDetailListItem";
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
    pub fn new(dispatcher: Arc<Mutex<Dispatcher>>) -> Self {
        let widget: Self = glib::Object::new();

        let dispatcher = dispatcher.clone();
        let gesture = GestureClick::new();
        let widget_ = widget.clone();
        gesture.connect_released(move |gesture, _, _, _| {
            debug!("gesture start");
            gesture.set_state(gtk4::EventSequenceState::Claimed);
            dispatcher
                .lock()
                .expect("could not lock dispatcher")
                .dispatch_action(Action::Play(widget_.get_library_entry()));
            debug!("gesture start done");
        });
        widget.add_controller(gesture);

        widget
    }

    pub fn get_library_entry(&self) -> LibraryEntry {
        self.imp().library_entry.borrow().clone().expect("Wanted to get the library entry before setting it? o.O")
    }

    pub fn set_library_entry(&self, library_entry: LibraryEntry) {
        let mut entry = self.imp().library_entry.borrow_mut();
        *entry = Some(library_entry);
    }

    pub fn set_position(&self, position: u32) {
        self.imp().position.set_label(format!("#{}", position).as_str());
    }

    pub fn set_name(&self, name: &str) {
        self.imp().position.set_label(name);
    }

    pub fn set_state(&self, state: DetailListItemState) {
        match state {
            DetailListItemState::Playing => self.imp().icon.set_icon_name(Some("play")),
            DetailListItemState::Played => self.imp().icon.set_icon_name(Some("check")),
            DetailListItemState::None => self.imp().icon.set_icon_name(None)
        }
    }

    // pub fn connect_clicked(&self, callback: impl Fn(i32) + Send + Sync + 'static) {
    //     let gesture = GestureClick::new();
    //     gesture.connect_released(move |gesture, _, _, _| {
    //         gesture.set_state(gtk4::EventSequenceState::Claimed);
    //         callback(*self.imp().id.borrow());
    //     });
    //     self.add_controller(gesture);
    // }
}

pub enum DetailListItemState {
    Playing,
    Played,
    None
}
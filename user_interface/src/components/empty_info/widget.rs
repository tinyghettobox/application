use gtk4::{CompositeTemplate, glib};
use gtk4::glib::object_subclass;
use gtk4::glib::subclass::InitializingObject;
use gtk4::subclass::prelude::*;

#[derive(Default, CompositeTemplate)]
#[template(file = "./empty_info.ui")]
pub struct EmptyInfoWidgetImp {}

#[object_subclass]
impl ObjectSubclass for EmptyInfoWidgetImp {
    const NAME: &'static str = "MupiboxEmptyInfo";
    type Type = EmptyInfoWidget;
    type ParentType = gtk4::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for EmptyInfoWidgetImp {}
impl WidgetImpl for EmptyInfoWidgetImp {}
impl BoxImpl for EmptyInfoWidgetImp {}

glib::wrapper! {
    pub struct EmptyInfoWidget(ObjectSubclass<EmptyInfoWidgetImp>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl EmptyInfoWidget {
    pub fn new() -> Self {
        glib::Object::new()
    }
}

use gtk4::{CompositeTemplate, glib, Widget};
use gtk4::glib::subclass::InitializingObject;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::subclass::prelude::ObjectSubclass;

#[derive(Default, CompositeTemplate)]
#[template(file = "./content.ui")]
pub struct ContentWidgetImp {
    #[template_child]
    pub view: TemplateChild<gtk4::Stack>
}

#[glib::object_subclass]
impl ObjectSubclass for ContentWidgetImp {
    const NAME: &'static str = "MupiboxContent";
    type Type = ContentWidget;
    type ParentType = gtk4::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for ContentWidgetImp {}
impl WidgetImpl for ContentWidgetImp {}
impl BoxImpl for ContentWidgetImp {}

glib::wrapper! {
    pub struct ContentWidget(ObjectSubclass<ContentWidgetImp>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl ContentWidget {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn append_child(&self, name: &'static str, widget: &impl IsA<Widget>) {
        self.imp().view.add_named(widget, Some(name));
    }

    pub fn set_active_child(&self, name: String) {
        self.imp().view.set_visible_child_name(&name);
    }
}
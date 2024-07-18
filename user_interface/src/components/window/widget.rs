use gtk4::{CompositeTemplate, gio, glib, prelude::*, TemplateChild, Widget};
use gtk4::glib::{object_subclass};
use gtk4::subclass::prelude::*;
use gtk4::subclass::prelude::ObjectSubclassIsExt;

#[derive(Default, CompositeTemplate)]
#[template(file = "./window.ui")]
pub struct WindowWidgetImp {
    #[template_child]
    pub container: TemplateChild<gtk4::Box>,
}


#[object_subclass]
impl ObjectSubclass for WindowWidgetImp {
    const NAME: &'static str = "MupiboxWindow";
    type Type = WindowWidget;
    type ParentType = gtk4::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for WindowWidgetImp {}
impl WidgetImpl for WindowWidgetImp {}
impl WindowImpl for WindowWidgetImp {}
impl ApplicationWindowImpl for WindowWidgetImp {}

glib::wrapper! {
    pub struct WindowWidget(ObjectSubclass<WindowWidgetImp>)
        @extends gtk4::ApplicationWindow, gtk4::Window, gtk4::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk4::Accessible, gtk4::Buildable,
                    gtk4::ConstraintTarget, gtk4::Native, gtk4::Root, gtk4::ShortcutManager;
}

impl WindowWidget {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn add_child(&self, widget: &impl IsA<Widget>) {
        self.imp().container.get().append(widget);
    }
}
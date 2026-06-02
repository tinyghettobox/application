use gtk4::glib::object_subclass;
use gtk4::prelude::{BoxExt, ButtonExt, IsA};
use gtk4::subclass::prelude::*;
use gtk4::{glib, CompositeTemplate, TemplateChild, Widget};

#[derive(Default, CompositeTemplate)]
#[template(file = "./notification.ui")]
pub struct NotificationWidgetImp {
    #[template_child]
    close_button: TemplateChild<gtk4::Button>,
    #[template_child]
    revealer: TemplateChild<gtk4::Revealer>,
    #[template_child]
    notification_box: TemplateChild<gtk4::Box>,
    #[template_child]
    notification_label: TemplateChild<gtk4::Label>,
    #[template_child]
    container: TemplateChild<gtk4::Box>,
}

#[object_subclass]
impl ObjectSubclass for NotificationWidgetImp {
    const NAME: &'static str = "TinyGhettoBoxNotification";
    type Type = NotificationWidget;
    type ParentType = gtk4::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for NotificationWidgetImp {}
impl WidgetImpl for NotificationWidgetImp {}
impl BoxImpl for NotificationWidgetImp {}

glib::wrapper! {
    pub struct NotificationWidget(ObjectSubclass<NotificationWidgetImp>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl NotificationWidget {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn is_revealed(&self) -> bool {
        self.imp().revealer.reveals_child()
    }

    pub fn show_notification(&self, message: &str) {
        self.imp().notification_label.set_label(message);
        self.imp().revealer.set_reveal_child(true);
    }

    pub fn hide_notification(&self) {
        self.imp().revealer.set_reveal_child(false);
    }

    pub fn connect_close_clicked(&self, callback: impl Fn(&gtk4::Button) + 'static) {
        self.imp().close_button.connect_clicked(callback);
    }

    pub fn add_child(&self, widget: &impl IsA<Widget>) {
        self.imp().container.get().append(widget);
    }
}

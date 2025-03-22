use crate::util::memory_subscriber::LogMessage;
use gtk4::gio::ListStore;
use gtk4::glib::{object_subclass, BoxedAnyObject};
use gtk4::prelude::{BoxExt, ButtonExt, Cast, CastNone, IsA, ListItemExt, WidgetExt};
use gtk4::subclass::prelude::*;
use gtk4::{
    glib, CompositeTemplate, ListItem, NoSelection, Orientation, SignalListItemFactory,
    TemplateChild, Widget,
};

#[derive(Default, CompositeTemplate)]
#[template(file = "./log_overlay.ui")]
pub struct LogOverlayWidgetImp {
    #[template_child]
    close_button: TemplateChild<gtk4::Button>,
    #[template_child]
    list: TemplateChild<gtk4::ListView>,
    #[template_child]
    overlay: TemplateChild<gtk4::Box>,
    #[template_child]
    container: TemplateChild<gtk4::Box>,
}

#[object_subclass]
impl ObjectSubclass for LogOverlayWidgetImp {
    const NAME: &'static str = "TinyGhettoBoxLogOverlay";
    type Type = LogOverlayWidget;
    type ParentType = gtk4::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for LogOverlayWidgetImp {}
impl WidgetImpl for LogOverlayWidgetImp {}
impl BoxImpl for LogOverlayWidgetImp {}

glib::wrapper! {
    pub struct LogOverlayWidget(ObjectSubclass<LogOverlayWidgetImp>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl LogOverlayWidget {
    pub fn new() -> Self {
        glib::Object::new()
    }
    pub fn set_factory(&self) {
        let factory = SignalListItemFactory::new();

        factory.connect_setup(move |_, list_item| {
            let item = list_item
                .downcast_ref::<ListItem>()
                .expect("Needs to be ListItem");

            let wrapper = gtk4::Box::new(Orientation::Horizontal, 8);
            wrapper.set_can_focus(false);
            wrapper.set_can_target(false);

            let time = gtk4::Label::new(None);
            time.add_css_class("time");
            wrapper.append(&time);
            let level = gtk4::Label::new(None);
            level.add_css_class("level");
            wrapper.append(&level);
            let target = gtk4::Label::new(None);
            target.add_css_class("target");
            wrapper.append(&target);
            let message = gtk4::Label::new(None);
            message.add_css_class("message");
            wrapper.append(&message);

            item.set_child(Some(&wrapper));
        });

        factory.connect_bind(move |_, list_item| {
            let list_item = list_item
                .downcast_ref::<ListItem>()
                .expect("could not downcast to list item");
            let boxed = list_item
                .item()
                .and_downcast::<BoxedAnyObject>()
                .expect("Could not downcast to boxed any object");
            let log_entry = boxed.borrow::<LogMessage>();

            let wrapper = list_item
                .child()
                .and_downcast::<gtk4::Box>()
                .expect("could not downcast to widget");

            let time = wrapper
                .first_child()
                .unwrap()
                .downcast::<gtk4::Label>()
                .unwrap();
            time.set_label(&log_entry.time);
            let level = time
                .next_sibling()
                .unwrap()
                .downcast::<gtk4::Label>()
                .unwrap();
            level.add_css_class(log_entry.level.to_lowercase().as_str());
            level.set_label(&log_entry.level);
            let target = level
                .next_sibling()
                .unwrap()
                .downcast::<gtk4::Label>()
                .unwrap();
            // target.set_label(&log_entry.target);
            let message = target
                .next_sibling()
                .unwrap()
                .downcast::<gtk4::Label>()
                .unwrap();
            message.set_label(&log_entry.message);
        });

        factory.connect_unbind(|_, list_item| {
            let wrapper = list_item
                .downcast_ref::<ListItem>()
                .expect("Needs to be ListItem")
                .child()
                .and_downcast::<gtk4::Box>()
                .expect("could not downcast to label");

            let time = wrapper
                .first_child()
                .unwrap()
                .downcast::<gtk4::Label>()
                .unwrap();
            time.set_label("");
            let level = time
                .next_sibling()
                .unwrap()
                .downcast::<gtk4::Label>()
                .unwrap();
            level.set_label("");
            let target = level
                .next_sibling()
                .unwrap()
                .downcast::<gtk4::Label>()
                .unwrap();
            target.set_label("");
            let message = level
                .next_sibling()
                .unwrap()
                .downcast::<gtk4::Label>()
                .unwrap();
            message.set_label("");
        });

        self.imp().list.set_factory(Some(&factory));
    }

    pub fn set_list_store(&self, store: ListStore) {
        self.imp().list.set_can_target(false);
        self.imp().list.set_can_focus(false);
        self.imp()
            .list
            .set_model(Some(&NoSelection::new(Some(store))));
    }

    pub fn connect_close_clicked(&self, callback: impl Fn(&gtk4::Button) + 'static) {
        self.imp().close_button.connect_clicked(callback);
    }

    pub fn set_visible(&self, visible: bool) {
        self.imp().overlay.set_visible(visible);
    }

    pub fn add_child(&self, widget: &impl IsA<Widget>) {
        self.imp().container.get().append(widget);
    }
}

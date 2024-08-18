use gtk4::{glib, prelude::*, CompositeTemplate, TemplateChild};
use gtk4::gdk::Texture;
use gtk4::glib::{object_subclass};
use gtk4::glib::Bytes;
use gtk4::subclass::prelude::*;
use gtk4::subclass::prelude::ObjectSubclassIsExt;
use tracing::warn;

#[derive(Default, CompositeTemplate)]
#[template(file = "./navbar.ui")]
pub struct NavbarWidgetImp {
    #[template_child]
    pub wrapper: TemplateChild<gtk4::Box>,
    #[template_child]
    pub image: TemplateChild<gtk4::Picture>,
    #[template_child]
    pub label: TemplateChild<gtk4::Label>,
    #[template_child]
    pub back_button: TemplateChild<gtk4::Button>
}


#[object_subclass]
impl ObjectSubclass for NavbarWidgetImp {
    const NAME: &'static str = "TinyGhettoBoxNavbar";
    type Type = NavbarWidget;
    type ParentType = gtk4::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for NavbarWidgetImp {}
impl WidgetImpl for NavbarWidgetImp {}
impl BoxImpl for NavbarWidgetImp {}

glib::wrapper! {
    pub struct NavbarWidget(ObjectSubclass<NavbarWidgetImp>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl NavbarWidget {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn set_visibility(&self, visible: bool) {
        self.imp().wrapper.set_visible(visible);
    }

    pub fn set_image(&self, buffer: Option<Vec<u8>>) {
        let paintable = buffer.and_then(|buffer| {
            let bytes = Bytes::from(&buffer);
            match Texture::from_bytes(&bytes) {
                Ok(texture) => Some(texture),
                Err(error) => {
                    warn!("Failed to load texture: {}", error);
                    None
                }
            }
        });
        self.imp().image.set_paintable(paintable.as_ref());
    }

    pub fn set_name(&self, name: String) {
        self.imp().label.set_label(&name);
    }

    pub fn connect_back_clicked(&self, callback: impl Fn(&gtk4::Button) + 'static) {
        self.imp().back_button.connect_clicked(callback);
    }
}
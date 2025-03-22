use gtk4::gdk::Texture;
use gtk4::glib::subclass::InitializingObject;
use gtk4::glib::{object_subclass, Bytes};
use gtk4::prelude::{ButtonExt, GestureExt, WidgetExt};
use gtk4::subclass::prelude::*;
use gtk4::{glib, CompositeTemplate, GestureClick, GestureLongPress};
use tracing::warn;

#[derive(Default, CompositeTemplate)]
#[template(file = "./tile_list_item.ui")]
pub struct TileListItemWidgetImp {
    #[template_child]
    pub wrapper: TemplateChild<gtk4::Box>,
    #[template_child]
    pub image: TemplateChild<gtk4::Picture>,
    #[template_child]
    pub label: TemplateChild<gtk4::Label>,
    #[template_child]
    pub play_button: TemplateChild<gtk4::Button>,
}

#[object_subclass]
impl ObjectSubclass for TileListItemWidgetImp {
    const NAME: &'static str = "TinyGhettoBoxTileListItem";
    type Type = TileListItemWidget;
    type ParentType = gtk4::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for TileListItemWidgetImp {}
impl WidgetImpl for TileListItemWidgetImp {}
impl BoxImpl for TileListItemWidgetImp {}

glib::wrapper! {
    pub struct TileListItemWidget(ObjectSubclass<TileListItemWidgetImp>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl TileListItemWidget {
    pub fn new() -> Self {
        glib::Object::new()
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

    pub fn set_playing(&self, playing: bool) {
        if playing {
            self.imp().wrapper.add_css_class("playing");
        } else {
            self.imp().wrapper.remove_css_class("playing");
        }
    }

    pub fn connect_clicked(&self, callback: impl Fn() + Send + Sync + 'static) {
        // let gesture = GestureLongPress::new();
        // gesture.set_delay_factor(0.0);
        let gesture = GestureClick::new();
        gesture.connect_released(move |gesture, _, _, _| {
            gesture.set_state(gtk4::EventSequenceState::Claimed);
            callback();
        });
        self.imp().wrapper.add_controller(gesture);
    }

    pub fn connect_play_clicked(&self, callback: impl Fn() + Send + Sync + 'static) {
        self.imp().play_button.connect_clicked(move |_| callback());
    }
}

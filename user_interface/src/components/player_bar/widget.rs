use gtk4::{CompositeTemplate, gio, glib};
use gtk4::gdk::Texture;
use gtk4::glib::{Bytes, object_subclass, Propagation};
use gtk4::glib::subclass::InitializingObject;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use tracing::warn;

#[derive(Default, CompositeTemplate)]
#[template(file = "./player_bar.ui")]
pub struct PlayerBarWidgetImp {
    #[template_child]
    wrapper: TemplateChild<gtk4::Box>,
    #[template_child]
    progress_bar: TemplateChild<gtk4::Scale>,
    #[template_child]
    image: TemplateChild<gtk4::Picture>,
    #[template_child]
    track_name: TemplateChild<gtk4::Label>,
    #[template_child]
    folder_name: TemplateChild<gtk4::Label>,
    #[template_child]
    back_button: TemplateChild<gtk4::Button>,
    #[template_child]
    play_toggle_button: TemplateChild<gtk4::Button>,
    #[template_child]
    forward_button: TemplateChild<gtk4::Button>,
    #[template_child]
    volume_button: TemplateChild<gtk4::ScaleButton>,
}

#[object_subclass]
impl ObjectSubclass for PlayerBarWidgetImp {
    const NAME: &'static str = "TinyGhettoBoxPlayerBar";
    type Type = PlayerBarWidget;
    type ParentType = gtk4::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template()
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for PlayerBarWidgetImp {}
impl WidgetImpl for PlayerBarWidgetImp {}
impl BoxImpl for PlayerBarWidgetImp {}

glib::wrapper! {
    pub struct PlayerBarWidget(ObjectSubclass<PlayerBarWidgetImp>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl PlayerBarWidget {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn set_visibility(&self, visible: bool) {
        self.imp().wrapper.set_visible(visible);
    }

    pub fn set_progress(&self, progress: f64) {
        self.imp().progress_bar.adjustment().set_value(progress);
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

    pub fn set_track_name(&self, name: String) {
        self.imp().track_name.set_label(&name);
    }

    pub fn set_folder_name(&self, name: String) {
        self.imp().folder_name.set_label(&name);
    }

    pub fn set_paused(&self, paused: bool) {
        let icon_name = if paused {
            "play"
        } else {
            "pause"
        };
        self.imp().play_toggle_button.set_icon_name(icon_name);
    }

    pub fn set_volume(&self, volume: f64) {
        let adjustment = self.imp().volume_button.adjustment();
        adjustment.set_value(volume);
        self.imp().volume_button.set_adjustment(&adjustment);
    }

    pub fn connect_seek(&self, callback: impl Fn(f64) + 'static) {
        self.imp().progress_bar.connect_change_value(move |_scale, _scroll_type, new_value| {
            callback(new_value);
            Propagation::Proceed
        });
    }

    pub fn connect_back_clicked(&self, callback: impl Fn() + 'static) {
        self.imp().back_button.connect_clicked(move |_| callback());
    }

    pub fn connect_play_toggle_clicked(&self, callback: impl Fn() + 'static) {
        self.imp().play_toggle_button.connect_clicked(move |_| callback());
    }

    pub fn connect_forward_clicked(&self, callback: impl Fn() + 'static) {
        self.imp().forward_button.connect_clicked(move |_| callback());
    }

    pub fn connect_volume_change(&self, callback: impl Fn(f64) + 'static) {
        self.imp().volume_button.connect_value_changed(move |_scale, value| {
            callback(value)
        });
    }
}

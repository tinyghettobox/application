use gtk4::glib::object_subclass;
use gtk4::prelude::{BoxExt, EventControllerExt, GestureExt, IsA, WidgetExt};
use gtk4::subclass::prelude::*;
use gtk4::{glib, CompositeTemplate, GestureClick, PropagationPhase, Widget};

#[derive(Default, CompositeTemplate)]
#[template(file = "./shutdown_timer.ui")]
pub struct ShutdownTimerWidgetImp {
    #[template_child]
    container: TemplateChild<gtk4::Box>,
}

#[object_subclass]
impl ObjectSubclass for ShutdownTimerWidgetImp {
    const NAME: &'static str = "TinyGhettoBoxShutdownTimer";
    type Type = ShutdownTimerWidget;
    type ParentType = gtk4::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for ShutdownTimerWidgetImp {}
impl WidgetImpl for ShutdownTimerWidgetImp {}
impl BoxImpl for ShutdownTimerWidgetImp {}

glib::wrapper! {
    pub struct ShutdownTimerWidget(ObjectSubclass<ShutdownTimerWidgetImp>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl ShutdownTimerWidget {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn add_child(&self, widget: &impl IsA<Widget>) {
        self.imp().container.get().append(widget);
    }

    pub fn connect_clicked(&self, callback: impl Fn() + Send + Sync + 'static) {
        let gesture = GestureClick::new();
        gesture.set_propagation_phase(PropagationPhase::Capture);
        gesture.connect_released(move |gesture, _, _, _| {
            gesture.set_state(gtk4::EventSequenceState::None);
            callback();
        });
        self.imp().container.add_controller(gesture);
    }

    pub fn set_forwards_clicks(&self, forwards: bool) {
        self.imp().container.get().set_can_target(forwards);
    }
}

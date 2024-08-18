use std::time::Duration;

use gtk4::{CompositeTemplate, glib, PropagationPhase, TemplateChild, Widget};
use gtk4::glib::object_subclass;
use gtk4::prelude::{BoxExt, DrawingAreaExtManual, EventControllerExt, IsA, WidgetExt};
use gtk4::subclass::prelude::*;

#[derive(Default, CompositeTemplate)]
#[template(file = "./ripple.ui")]
pub struct RippleWidgetImp {
    #[template_child]
    pub container: TemplateChild<gtk4::Box>,
    #[template_child]
    pub canvas: TemplateChild<gtk4::DrawingArea>,
}


#[object_subclass]
impl ObjectSubclass for RippleWidgetImp {
    const NAME: &'static str = "TinyGhettoBoxRipple";
    type Type = RippleWidget;
    type ParentType = gtk4::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for RippleWidgetImp {}
impl WidgetImpl for RippleWidgetImp {}
impl BoxImpl for RippleWidgetImp {}


glib::wrapper! {
    pub struct RippleWidget(ObjectSubclass<RippleWidgetImp>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl RippleWidget {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn add_child(&self, widget: &impl IsA<Widget>) {
        self.imp().container.get().append(widget);
    }

    pub fn start_ripple_drawing(&self) {
        let ripple_data = std::rc::Rc::new(std::cell::RefCell::new(None));
        let ripple_data_ = ripple_data.clone();

        self.imp().canvas.set_draw_func(move |_, cr, _width, _height| {
            let data = ripple_data_.borrow();
            if let Some((x, y, radius, alpha)) = *data {
                cr.set_source_rgba(200.0, 200.0, 200.0, alpha);
                cr.arc(x, y, radius, 0.0, 2.0 * std::f64::consts::PI);
                cr.fill().expect("could draw fill circle");
            }
        });

        let canvas = self.imp().canvas.clone();
        canvas.set_can_target(false);

        let gesture = gtk4::GestureClick::new();
        gesture.set_propagation_phase(PropagationPhase::Capture);
        gesture.connect_pressed(move |_gesture, _, x, y| {
            let start_time = std::time::Instant::now();
            let ripple_data = ripple_data.clone();
            let canvas = canvas.clone();
            glib::timeout_add_local(Duration::from_millis(16), move || {
                let elapsed = start_time.elapsed().as_secs_f64();
                let radius = 5.0 + elapsed * 30.0;
                let alpha = 1.0 - elapsed / 0.6;
                if alpha > 0.0 {
                    *ripple_data.borrow_mut() = Some((x, y, radius, alpha));
                    canvas.queue_draw();
                    glib::ControlFlow::Continue
                } else {
                    *ripple_data.borrow_mut() = None;
                    canvas.queue_draw();
                    glib::ControlFlow::Break
                }
            });
        });
        self.imp().container.add_controller(gesture);
    }
}
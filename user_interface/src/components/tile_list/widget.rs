use gtk4::{Adjustment, CompositeTemplate, glib, Widget};
use gtk4::glib::object_subclass;
use gtk4::glib::subclass::InitializingObject;
use gtk4::prelude::{AdjustmentExt, GridExt, IsA, WidgetExt};
use gtk4::subclass::prelude::*;
use tracing::warn;

#[derive(Default, CompositeTemplate)]
#[template(file = "./tile_list.ui")]
pub struct TileListWidgetImp {
    #[template_child]
    pub grid: TemplateChild<gtk4::Grid>,
    #[template_child]
    pub scroll_window: TemplateChild<gtk4::ScrolledWindow>,
}

#[object_subclass]
impl ObjectSubclass for TileListWidgetImp {
    const NAME: &'static str = "TinyGhettoBoxTileList";
    type Type = TileListWidget;
    type ParentType = gtk4::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for TileListWidgetImp {}
impl WidgetImpl for TileListWidgetImp {}
impl BoxImpl for TileListWidgetImp {}

glib::wrapper! {
    pub struct TileListWidget(ObjectSubclass<TileListWidgetImp>)
        @extends gtk4::Box, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl TileListWidget {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn remove_children(&self) {
        tracing::debug!("Calling remove");
        let grid = self.imp().grid.get();
        let mut child = grid.first_child();
        while let Some(widget) = child.as_ref() {
            let next_child = widget.next_sibling();
            grid.remove(widget);
            child = next_child;
        }
    }

    pub fn set_children(&self, children: &Vec<impl IsA<Widget>>, start_row: i32, start_column: i32) {
        let grid = self.imp().grid.get();

        for (index, child) in children.iter().enumerate() {
            let column = start_column + index as i32 % 3;
            let row = start_row + index as i32 / 3;

            grid.attach(child, column, row, 1, 1);
        }
    }

    pub fn connect_scroll_end<C>(&self, handle_scroll_end: C)
    where
        C: Fn() + 'static,
    {
        let grid = self.imp().grid.clone();
        let on_scroll = move |adjustment: &Adjustment| {
            let scroll_position = adjustment.value() + adjustment.page_size();
            let scroll_height = adjustment.upper();

            let last_child_bounds = {
                let rect = grid.last_child().and_then(|child| child.compute_bounds(&child));
                match rect {
                    Some(rect) => rect,
                    None => {
                        warn!("Could not determine last child position");
                        return;
                    }
                }
            };

            if scroll_height - scroll_position < last_child_bounds.height() as f64 {
                handle_scroll_end();
            }
        };

        self.imp().scroll_window.vadjustment().connect_value_changed(on_scroll);
    }
}

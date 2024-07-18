use gtk4::{CompositeTemplate, glib};
use gtk4::glib::object_subclass;
use gtk4::glib::subclass::InitializingObject;
use gtk4::prelude::{GridExt, WidgetExt};
use gtk4::subclass::prelude::*;

use crate::components::tile_list_item::TileListItemComponent;

#[derive(Default, CompositeTemplate)]
#[template(file = "./tile_list.ui")]
pub struct TileListWidgetImp {
    #[template_child]
    pub grid: TemplateChild<gtk4::Grid>,
}

#[object_subclass]
impl ObjectSubclass for TileListWidgetImp {
    const NAME: &'static str = "MupiboxTileList";
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
        let grid = self.imp().grid.get();
        let mut child = grid.first_child();
        while let Some(widget) = child.as_ref() {
            let next_child = widget.next_sibling();
            grid.remove(widget);
            child = next_child;
        }
    }

    pub fn set_children(&self, children: &Vec<TileListItemComponent>) {
        let grid = self.imp().grid.get();

        for (index, child) in children.iter().enumerate() {
            let column = index as i32 % 3;
            let row = index as i32 / 3;

            grid.attach(&child.widget, column, row, 1, 1);
        }
    }
}

use std::sync::{Arc};
use std::sync::Mutex;

use gtk4::prelude::IsA;
use gtk4::Widget;

pub use window::WindowComponent;

use crate::state::{Dispatcher, EventHandler, State};

mod window;
mod navbar;
mod content;
mod tile_list;
mod tile_list_item;
mod detail_list;
mod detail_list_item;
mod list_item_factory;
mod empty_info;
mod player_bar;

pub type Children = Vec<Arc<Mutex<Box<dyn EventHandler>>>>;

pub trait Component<P>: EventHandler {
    fn new(state: Arc<Mutex<State>>, dispatcher: Arc<Mutex<Dispatcher>>, params: P) -> Self;
    fn render(state: Arc<Mutex<State>>, dispatcher: Arc<Mutex<Dispatcher>>, params: P) -> (impl IsA<Widget>, Children);
    fn update(&mut self);
    fn get_widget(&self) -> impl IsA<Widget>;
}

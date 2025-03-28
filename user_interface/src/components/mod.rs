use std::sync::{Arc, Mutex};

use gtk4::prelude::IsA;
use gtk4::Widget;

pub use window::WindowComponent;

use crate::state::{Dispatcher, EventHandler, State};

mod content;
mod detail_list;
mod detail_list_item;
mod empty_info;
mod log_overlay;
mod navbar;
mod player_bar;
mod ripple;
mod shutdown_timer;
mod tile_list;
mod tile_list_item;
mod window;

pub type Children = Vec<Arc<Mutex<Box<dyn EventHandler>>>>;

pub trait Component<P>: EventHandler {
    fn new(state: Arc<Mutex<State>>, dispatcher: Arc<Mutex<Dispatcher>>, params: P) -> Self;
    fn render(
        state: Arc<Mutex<State>>,
        dispatcher: Arc<Mutex<Dispatcher>>,
        params: P,
    ) -> (impl IsA<Widget>, Children);
    fn update(&mut self);
    fn get_widget(&self) -> impl IsA<Widget>;
}

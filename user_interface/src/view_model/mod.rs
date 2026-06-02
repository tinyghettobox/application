pub mod content;
pub mod display_timer;
pub mod logs;
pub mod messages;
pub mod navbar;
pub mod playbar;

use slint::Weak;
use crate::AppWindow;

pub fn update_ui<F>(ui: &Weak<AppWindow>, f: F)
where
    F: FnOnce(&AppWindow) + Send + 'static,
{
    let ui = ui.clone();
    slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui.upgrade() {
            f(&ui);
        }
    }).ok();
}
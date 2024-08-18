use std::env::current_dir;
use std::path::{Path, PathBuf};

use image::EncodableLayout;
use minifb::{Window, WindowOptions};
use tracing::debug;

pub struct Splash {
    window: Window
}

impl Splash {
    pub fn new() -> Option<Self> {
        let image_path = match get_image_path() {
            Some(path) => path,
            None => {
                debug!("Did not find a splash image");
                return None
            }
        };
        let img = image::open(image_path).expect("could not open splash image");
        let buffer = img.to_rgb8()
            .pixels()
            .map(|p| {
                let rgba = p.0;
                (rgba[0] as u32) << 16 | (rgba[1] as u32) << 8 | (rgba[2] as u32)
            })
            .collect::<Vec<u32>>();

        let mut window = Window::new("splash", img.width() as usize, img.height() as usize, WindowOptions {
            borderless: true,
            title: false,
            topmost: true,
            ..WindowOptions::default()
        }).expect("Could not create window");
        window.update_with_buffer(&buffer, img.width() as usize, img.height() as usize).expect("Could not update window with buffer");

        Some(
            Splash {
                window
            }
        )
    }
}

fn get_image_path() -> Option<PathBuf> {
    let boot_path = Path::new("/boot/splash-startup.png");
    if boot_path.exists() {
        return Some(boot_path.to_owned());
    }
    let resource_path = Path::new("./user_interface/resources/splash-startup.png");
    if resource_path.exists() {
        return Some(resource_path.to_owned());
    }
    debug!("Path {:?}", current_dir());
    None
}

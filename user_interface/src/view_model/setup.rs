use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use slint::{ComponentHandle, Image, Rgba8Pixel, SharedPixelBuffer, Weak};

use crate::model::{Field, State, actions::Action};
use crate::view_model::update_ui;
use crate::{AppWindow, Setup};

pub struct SetupVM {
    ui: Weak<AppWindow>,
    state: State,
}

/// How many times `enable_ap.sh` may be attempted (1 initial + 2 retries).
const AP_MAX_ATTEMPTS: u32 = 3;

/// Raw RGBA pixel data for the QR code image, safe to send across threads.
struct QrPixels {
    width: u32,
    height: u32,
    data: Vec<u8>,
}

impl SetupVM {
    pub fn new(ui: Weak<AppWindow>, state: State, initial_setup_complete: bool) -> Self {
        let vm = SetupVM { ui, state };
        vm.setup_state_listeners(initial_setup_complete);
        vm
    }

    pub fn setup_state_listeners(&self, initial_setup_complete: bool) {
        let ui_weak = self.ui.clone();
        let state_clone = self.state.clone();
        // Pre-seed as true when setup was already complete at startup so that
        // LoadLibraryEntry(0) is not dispatched a second time here.
        let library_loaded = Arc::new(AtomicBool::new(initial_setup_complete));
        // Track whether the AP enable thread has already been started so it
        // fires at most once (the thread itself retries up to AP_MAX_ATTEMPTS).
        let ap_enable_started = Arc::new(AtomicBool::new(false));

        self.state.subscribe(move |changes| {
            if !changes.iter().any(|f| matches!(f, Field::system_config(_))) {
                return;
            }

            let system_config = state_clone.system_config();
            let Some(config) = system_config else { return };

            let setup_complete = config.setup_complete;
            let ap_password = config.ap_password.clone();

            if setup_complete {
                update_ui(&ui_weak, move |ui| {
                    ui.global::<Setup>().set_visible(false);
                });
                if !library_loaded.swap(true, Ordering::SeqCst) {
                    state_clone.dispatch(Action::LoadLibraryEntry(0));
                }
                return;
            }

            // Build QR pixel data on this thread (no Slint types involved yet).
            let qr_pixels = build_wifi_qr_pixels(&ap_password);

            {
                let ap_password = ap_password.clone();
                update_ui(&ui_weak, move |ui| {
                    let setup = ui.global::<Setup>();
                    setup.set_visible(true);
                    setup.set_ap_password(ap_password.into());
                    if let Some(pixels) = qr_pixels {
                        let mut buf = SharedPixelBuffer::<Rgba8Pixel>::new(pixels.width, pixels.height);
                        buf.make_mut_bytes().copy_from_slice(&pixels.data);
                        setup.set_qr_code(Image::from_rgba8(buf));
                    }
                });
            }

            // Enable the access point once; the thread retries up to AP_MAX_ATTEMPTS times.
            if !ap_enable_started.swap(true, Ordering::SeqCst) {
                std::thread::spawn(move || enable_ap_with_retries(&ap_password));
            }
        });
    }
}

/// Runs `enable_ap.sh on tinyghettobox <password>`, retrying up to `AP_MAX_ATTEMPTS` times.
fn enable_ap_with_retries(ap_password: &str) {
    for attempt in 1..=AP_MAX_ATTEMPTS {
        tracing::info!("Enabling WiFi access point (attempt {}/{})", attempt, AP_MAX_ATTEMPTS);
        match std::process::Command::new("/srv/tinyghettobox/enable_ap.sh")
            .args(["on", "tinyghettobox", ap_password])
            .output()
        {
            Ok(out) if out.status.success() => {
                tracing::info!("enable_ap.sh succeeded");
                return;
            }
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                let stdout = String::from_utf8_lossy(&out.stdout);
                let msg = if !stderr.trim().is_empty() { stderr } else { stdout };
                tracing::error!("enable_ap.sh failed (exit {}): {}", out.status, msg.trim());
            }
            Err(e) => {
                tracing::error!("Failed to spawn enable_ap.sh: {}", e);
            }
        }
        if attempt < AP_MAX_ATTEMPTS {
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    }
}

/// Generates a QR code for `WIFI:S:tinyghettobox;T:WPA;P:<password>;;` and
/// returns raw RGBA pixel data (safe to send across threads).
fn build_wifi_qr_pixels(ap_password: &str) -> Option<QrPixels> {
    use qrcodegen::{QrCode, QrCodeEcc};

    let wifi_string = format!("WIFI:S:tinyghettobox;T:WPA;P:{};;", ap_password);
    let qr = QrCode::encode_text(&wifi_string, QrCodeEcc::Medium).ok()?;

    let module_size: usize = 6;
    let border: usize = 2;
    let total_modules = qr.size() as usize + border * 2;
    let pixel_size = total_modules * module_size;

    let mut data = vec![0u8; pixel_size * pixel_size * 4];

    for py in 0..pixel_size {
        for px in 0..pixel_size {
            let module_x = (px / module_size) as i32 - border as i32;
            let module_y = (py / module_size) as i32 - border as i32;

            let is_dark = module_x >= 0
                && module_y >= 0
                && module_x < qr.size()
                && module_y < qr.size()
                && qr.get_module(module_x, module_y);

            let (r, g, b) = if is_dark { (0u8, 0u8, 0u8) } else { (255u8, 255u8, 255u8) };
            let idx = (py * pixel_size + px) * 4;
            data[idx] = r;
            data[idx + 1] = g;
            data[idx + 2] = b;
            data[idx + 3] = 255;
        }
    }

    Some(QrPixels {
        width: pixel_size as u32,
        height: pixel_size as u32,
        data,
    })
}

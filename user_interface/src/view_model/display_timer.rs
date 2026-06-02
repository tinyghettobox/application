use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use slint::{ComponentHandle, Timer, TimerMode, Weak};
use tracing::{debug, info};

use crate::{
    model::{Field, State},
    view_model::update_ui,
    AppWindow, DisplayTimer,
};

pub struct DisplayTimerVM {
    _timer: Option<Timer>,
}

impl DisplayTimerVM {
    pub fn new(ui: Weak<AppWindow>, state: State) -> Self {
        let devices = Self::open_input_devices();
        if devices.is_empty() {
            info!("No accessible /dev/input/event* devices — display timer disabled");
            return Self { _timer: None };
        }

        let last_activity: Arc<Mutex<Instant>> = Arc::new(Mutex::new(Instant::now()));
        let display_off_secs: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));
        let shutdown_secs: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));

        {
            let display_off = display_off_secs.clone();
            let shutdown = shutdown_secs.clone();
            let conn = state.conn.clone();
            tokio::spawn(async move {
                if let Ok(Some(cfg)) = database::SystemConfigRepository::get(&conn).await {
                    *display_off.lock().unwrap() = cfg.display_off_timer as u64 * 60;
                    *shutdown.lock().unwrap() = cfg.sleep_timer as u64 * 60;
                }
            });
        }

        Self::spawn_input_monitors(devices, last_activity.clone());

        if let Some(ui) = ui.upgrade() {
            let state_ = state.clone();
            let last_activity_ = last_activity.clone();
            ui.global::<DisplayTimer>().on_wake_up(move || {
                *last_activity_.lock().unwrap() = Instant::now();
                state_.dispatch(crate::model::actions::Action::SetDisplayActive(true));
            });
        }

        {
            let ui_weak = ui.clone();
            state.subscribe(move |changes| {
                let active = changes.iter().find_map(|f| {
                    if let Field::display_active(v) = f { Some(*v) } else { None }
                });
                if let Some(active) = active {
                    update_ui(&ui_weak, move |ui| {
                        ui.global::<DisplayTimer>().set_display_active(active);
                    });
                }
            });
        }

        let timer = Timer::default();
        timer.start(TimerMode::Repeated, Duration::from_secs(1), {
            let state = state.clone();
            move || {
                let elapsed = last_activity.lock().unwrap().elapsed();
                let display_off = *display_off_secs.lock().unwrap();
                let shutdown = *shutdown_secs.lock().unwrap();
                let display_active = state.display_active();
                debug!(elapsed = ?elapsed, display_off, shutdown, display_active, "Timer tick");
                
                if display_off > 0 && elapsed >= Duration::from_secs(display_off) && display_active {
                    state.dispatch(crate::model::actions::Action::SetDisplayActive(false));
                }
                if shutdown > 0 && elapsed >= Duration::from_secs(shutdown) {
                    state.dispatch(crate::model::actions::Action::Shutdown);
                }
            }
        });

        Self { _timer: Some(timer) }
    }

    fn open_input_devices() -> Vec<std::fs::File> {
        let Ok(entries) = std::fs::read_dir("/dev/input") else {
            return vec![];
        };
        entries
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().starts_with("event"))
            .filter_map(|e| std::fs::File::open(e.path()).ok())
            .collect()
    }

    fn spawn_input_monitors(devices: Vec<std::fs::File>, last_activity: Arc<Mutex<Instant>>) {
        for file in devices {
            let last_activity = last_activity.clone();
            std::thread::Builder::new()
                .name("input-monitor".into())
                .spawn(move || {
                    use std::io::Read;
                    let mut file = file;
                    let mut buf = [0u8; 72];
                    loop {
                        match file.read(&mut buf) {
                            Ok(0) | Err(_) => break,
                            Ok(_) => { *last_activity.lock().unwrap() = Instant::now(); }
                        }
                    }
                })
                .ok();
        }
    }
}

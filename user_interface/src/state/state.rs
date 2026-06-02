use crate::util::memory_subscriber::LogMessage;
use database::model::library_entry::Model as LibraryEntry;
use database::{DatabaseConnection, LibraryEntryRepository, SystemConfigRepository};
use player::Progress;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tracing::warn;

pub struct State {
    pub started: bool,
    pub connection: DatabaseConnection,
    pub library_entry: LibraryEntry,
    pub active_view: String,
    pub start_playing: bool,
    pub playing_library_entry: Option<LibraryEntry>,
    pub playing_library_entry_path: Vec<i32>, // Path of the currently playing library entry from parent to current entry
    pub paused: bool,
    pub progress: Progress,
    pub volume: f64,
    pub max_volume: f64,
    pub display_off_timeout: i64,
    pub sleep_timeout: i64,
    pub monitor_active: bool,
    pub last_activity: Instant,
    pub show_log_overlay: bool,
    pub messages: Arc<Mutex<Vec<LogMessage>>>,
    pub log_level: String,
}

impl State {
    pub async fn new(
        connection: DatabaseConnection,
        messages: Arc<Mutex<Vec<LogMessage>>>,
    ) -> Self {
        let system_config = SystemConfigRepository::get(&connection)
            .await
            .unwrap()
            .unwrap();
        let library_entry = LibraryEntryRepository::get(&connection, 0)
            .await
            .expect("Failed to get root library entry")
            .expect("No root library entry found");

        let active_view = if library_entry
            .children
            .as_ref()
            .map(|children| children.len())
            .unwrap_or(0)
            > 0
        {
            "tile_list".to_string()
        } else {
            "empty_info".to_string()
        };

        // Pulseaudio is storing data in /var/lib/pulse which is mounted on tmpfs and loses data on
        // reboot. We need to restore the max volume to the system config value on startup.
        if cfg!(target_os = "linux") {
            let result = std::process::Command::new("pactl")
                .arg("set-sink-volume")
                .arg("@DEFAULT_SINK@")
                .arg(format!("{}%", system_config.max_volume))
                .spawn();
            if let Err(error) = result {
                warn!("Could not set max volume: {}", error);
            }
        }

        Self {
            connection,
            library_entry,
            active_view,
            volume: system_config.volume as f64 / 100.0,
            // max_volume: system_config.max_volume as f64 / 100.0,
            max_volume: 1.0,
            display_off_timeout: system_config.display_off_timer as i64,
            sleep_timeout: system_config.sleep_timer as i64,
            start_playing: false,
            playing_library_entry: None,
            playing_library_entry_path: vec![],
            paused: true,
            progress: Progress::default(),
            started: false,
            monitor_active: true,
            last_activity: Instant::now(),
            show_log_overlay: false,
            messages,
            log_level: "TRACE".to_string(),
        }
    }
}

use crate::util::memory_subscriber::LogMessage;
use database::model::library_entry::Model as LibraryEntry;
use database::{DatabaseConnection, LibraryEntryRepository, SystemConfigRepository};
use player::Progress;
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub struct State {
    pub started: bool,
    pub connection: DatabaseConnection,
    pub library_entry: LibraryEntry,
    pub active_view: String,
    pub playing_library_entry: Option<LibraryEntry>,
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

        Self {
            connection,
            library_entry,
            active_view,
            volume: system_config.volume as f64 / 100.0,
            max_volume: system_config.max_volume as f64 / 100.0,
            display_off_timeout: system_config.display_off_timer as i64,
            sleep_timeout: system_config.sleep_timer as i64,
            playing_library_entry: None,
            paused: true,
            progress: Progress::default(),
            started: false,
            monitor_active: true,
            last_activity: Instant::now(),
            show_log_overlay: false,
            messages,
        }
    }
}

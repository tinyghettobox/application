use crate::util::memory_subscriber::LogMessage;
use chrono::Utc;
use database::model::library_entry::Model as LibraryEntry;
use database::{DatabaseConnection, LibraryEntryRepository, SystemConfigRepository};
use player::Progress;
use std::sync::{Arc, Mutex};

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
    pub monitor_active: bool,
    pub last_activity: i64,
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
            playing_library_entry: None,
            paused: true,
            progress: Progress::default(),
            started: false,
            monitor_active: true,
            last_activity: Utc::now().timestamp(),
            show_log_overlay: false,
            messages,
        }
    }
}

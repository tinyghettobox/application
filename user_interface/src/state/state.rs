use database::{DatabaseConnection, LibraryEntryRepository};
use database::model::library_entry::Model as LibraryEntry;

pub struct State {
    pub connection: DatabaseConnection,
    pub library_entry: LibraryEntry,
    pub active_view: String,
    pub playing_library_entry: Option<LibraryEntry>,
    pub paused: bool,
    pub progress: f64,
}

impl State {
    pub async fn new(connection: DatabaseConnection) -> Self {
        let library_entry = LibraryEntryRepository::get(&connection, 0)
            .await
            .expect("Failed to get root library entry")
            .expect("No root library entry found");

        let active_view = if library_entry.children.as_ref().map(|children| children.len()).unwrap_or(0) > 0 {
            "tile_list".to_string()
        } else {
            "empty_info".to_string()
        };

        Self {
            connection,
            library_entry,
            active_view,
            playing_library_entry: None,
            paused: true,
            progress: 0.0
        }
    }
}
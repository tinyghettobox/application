use database::model::library_entry::Model as LibraryEntry;
use player::Progress;

mod append_log;
mod init_volume;
mod load_library_entries;
mod play_library_entry;
mod play_next;
mod play_prev;
mod seek_to;
mod set_played_at;
mod set_playing_track;
mod set_volume;
mod start_playing_library_entry;
mod toggle_play;
mod toggle_show_logs;
mod set_display_active;
mod shutdown;

#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
    pub timestamp: chrono::NaiveDateTime,
}

#[derive(Debug)]
pub enum Action {
    InitVolume,
    LoadLibraryEntry(i32),
    StartPlayingLibraryEntry(LibraryEntry),
    PlayLibraryEntry(LibraryEntry),
    TogglePlay(bool),
    SetProgress(Progress),
    SetPlayingTrack(Option<LibraryEntry>),
    SetPlayedAt(i32, bool),
    SeekTo(f64),
    PlayPrev,
    PlayNext,
    SetVolume(i32),
    AppendLog(LogEntry),
    ToggleShowLogs,
    SetDisplayActive(bool),
    Shutdown,
}

impl Action {
    pub fn should_log(&self) -> bool {
        !matches!(self, Action::SetProgress(_) | Action::AppendLog(_))
    }
}
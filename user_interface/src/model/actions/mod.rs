use database::model::library_entry::Model as LibraryEntry;
use player::Progress;

mod append_log;
mod clear_messages;
mod init_volume;
mod load_library_entries;
mod load_system_config;
mod play_library_entry;
mod play_next;
mod play_prev;
mod seek_to;
mod set_audio_status;
mod set_display_active;
mod set_played_at;
mod set_playing_track;
mod set_volume;
mod set_wifi_status;
mod shutdown;
mod start_playing_library_entry;
mod toggle_play;
mod toggle_show_logs;

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

/// Audio subsystem readiness state.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum AudioStatus {
    /// Waiting for audio daemon (PipeWire/PulseAudio) to become available.
    Initializing,
    /// Audio is ready; player has been initialised.
    Ready,
    /// Audio never became available within the timeout.
    Failed,
}

/// WiFi / network connectivity state.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum WifiStatus {
    /// System monitor has not yet determined connectivity.
    Initializing,
    /// Interface exists but is not associated to any AP.
    Disconnected,
    /// Connected; RSSI is poor (< −75 dBm).
    Weak(i8),
    /// Connected; RSSI is acceptable (−75 … −55 dBm).
    Good(i8),
    /// Connected; RSSI is strong (> −55 dBm).
    Strong(i8),
}

impl WifiStatus {
    /// True when the network is usable for playback attempts.
    pub fn is_network_available(&self) -> bool {
        matches!(self, WifiStatus::Weak(_) | WifiStatus::Good(_) | WifiStatus::Strong(_))
    }
}

#[derive(Debug)]
pub enum Action {
    InitVolume,
    LoadSystemConfig,
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
    ClearMessages,
    ToggleShowLogs,
    SetDisplayActive(bool),
    SetAudioStatus(AudioStatus),
    SetWifiStatus(WifiStatus),
    Shutdown,
}

impl Action {
    pub fn should_log(&self) -> bool {
        !matches!(self, Action::SetProgress(_) | Action::AppendLog(_) | Action::SetWifiStatus(_) | Action::LoadSystemConfig | Action::ClearMessages)
    }
}
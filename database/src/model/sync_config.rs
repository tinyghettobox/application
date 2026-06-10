use sea_orm::prelude::DateTimeUtc;
use serde::{Deserialize, Serialize};

/// Sync configuration for a library entry that is backed by a Spotify container
/// (artist, album, playlist, show).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SortBy {
    ReleaseDateDesc,
    ReleaseDateAsc,
    AlphaAsc,
    AlphaDesc,
    TrackNumberAsc,
    EpisodeRegexAsc,
    EpisodeRegexDesc,
    MostRecentlyPlayed,
    Manual,
}

impl SortBy {
    pub fn as_str(&self) -> &'static str {
        match self {
            SortBy::ReleaseDateDesc => "release_date_desc",
            SortBy::ReleaseDateAsc => "release_date_asc",
            SortBy::AlphaAsc => "alpha_asc",
            SortBy::AlphaDesc => "alpha_desc",
            SortBy::TrackNumberAsc => "track_number_asc",
            SortBy::EpisodeRegexAsc => "episode_regex_asc",
            SortBy::EpisodeRegexDesc => "episode_regex_desc",
            SortBy::MostRecentlyPlayed => "most_recently_played",
            SortBy::Manual => "manual",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "release_date_asc" => SortBy::ReleaseDateAsc,
            "alpha_asc" => SortBy::AlphaAsc,
            "alpha_desc" => SortBy::AlphaDesc,
            "track_number_asc" => SortBy::TrackNumberAsc,
            "episode_regex_asc" => SortBy::EpisodeRegexAsc,
            "episode_regex_desc" => SortBy::EpisodeRegexDesc,
            "most_recently_played" => SortBy::MostRecentlyPlayed,
            "manual" => SortBy::Manual,
            _ => SortBy::ReleaseDateDesc,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub library_entry_id: i32,
    pub sort_by: SortBy,
    pub sort_regex: Option<String>,
    /// 0 = manual only, 1 = daily, 7 = weekly, 30 = monthly
    pub sync_interval_days: i32,
    pub last_synced_at: Option<DateTimeUtc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSyncConfig {
    pub sort_by: SortBy,
    pub sort_regex: Option<String>,
    pub sync_interval_days: i32,
}

impl Default for CreateSyncConfig {
    fn default() -> Self {
        CreateSyncConfig {
            sort_by: SortBy::ReleaseDateDesc,
            sort_regex: None,
            sync_interval_days: 30,
        }
    }
}

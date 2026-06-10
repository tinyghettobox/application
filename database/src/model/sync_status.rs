use sea_orm::prelude::DateTimeUtc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SyncStatusKind {
    Pending,
    Running,
    Done,
    Error,
}

impl SyncStatusKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            SyncStatusKind::Pending => "pending",
            SyncStatusKind::Running => "running",
            SyncStatusKind::Done => "done",
            SyncStatusKind::Error => "error",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "running" => SyncStatusKind::Running,
            "done" => SyncStatusKind::Done,
            "error" => SyncStatusKind::Error,
            _ => SyncStatusKind::Pending,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub library_entry_id: i32,
    pub status: SyncStatusKind,
    pub items_done: i32,
    pub items_total: Option<i32>,
    pub error_message: Option<String>,
    pub started_at: Option<DateTimeUtc>,
    pub synced_at: Option<DateTimeUtc>,
}

use chrono::Utc;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, FromQueryResult, Statement, Value};

use crate::model::sync_status::{SyncStatus, SyncStatusKind};

pub struct SyncStatusRepository;

#[derive(FromQueryResult)]
struct SyncStatusRow {
    library_entry_id: i32,
    status: String,
    items_done: i32,
    items_total: Option<i32>,
    error_message: Option<String>,
    started_at: Option<String>,
    synced_at: Option<String>,
}

impl SyncStatusRow {
    fn into_model(self) -> SyncStatus {
        SyncStatus {
            library_entry_id: self.library_entry_id,
            status: SyncStatusKind::from_str(&self.status),
            items_done: self.items_done,
            items_total: self.items_total,
            error_message: self.error_message,
            started_at: self.started_at.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|d| d.with_timezone(&Utc))
            }),
            synced_at: self.synced_at.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s)
                    .ok()
                    .map(|d| d.with_timezone(&Utc))
            }),
        }
    }
}

impl SyncStatusRepository {
    pub async fn get(
        conn: &DatabaseConnection,
        library_entry_id: i32,
    ) -> Result<Option<SyncStatus>, DbErr> {
        let row = SyncStatusRow::find_by_statement(Statement::from_sql_and_values(
            sea_orm::DbBackend::Sqlite,
            "SELECT library_entry_id, status, items_done, items_total, error_message, started_at, synced_at FROM library_entry_sync_status WHERE library_entry_id = ?",
            vec![Value::Int(Some(library_entry_id))],
        ))
        .one(conn)
        .await?;

        Ok(row.map(SyncStatusRow::into_model))
    }

    /// Resets all entries stuck in `running` state back to `pending`.
    /// Call once at startup to recover from a crash mid-sync.
    pub async fn reset_stale_running(conn: &DatabaseConnection) -> Result<u64, DbErr> {
        let result = conn.execute(Statement::from_sql_and_values(
            sea_orm::DbBackend::Sqlite,
            "UPDATE library_entry_sync_status SET status = 'pending', items_done = 0, items_total = NULL WHERE status = 'running'",
            vec![],
        ))
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn set_pending(conn: &DatabaseConnection, library_entry_id: i32) -> Result<(), DbErr> {
        conn.execute(Statement::from_sql_and_values(
            sea_orm::DbBackend::Sqlite,
            r#"
            INSERT INTO library_entry_sync_status (library_entry_id, status, items_done, items_total, error_message, started_at, synced_at)
            VALUES (?, 'pending', 0, NULL, NULL, NULL, NULL)
            ON CONFLICT(library_entry_id) DO UPDATE SET status = 'pending', items_done = 0, items_total = NULL, error_message = NULL
            "#,
            vec![Value::Int(Some(library_entry_id))],
        ))
        .await?;
        Ok(())
    }

    pub async fn set_running(
        conn: &DatabaseConnection,
        library_entry_id: i32,
    ) -> Result<(), DbErr> {
        let now = Utc::now().to_rfc3339();
        conn.execute(Statement::from_sql_and_values(
            sea_orm::DbBackend::Sqlite,
            r#"
            INSERT INTO library_entry_sync_status (library_entry_id, status, items_done, items_total, error_message, started_at, synced_at)
            VALUES (?, 'running', 0, NULL, NULL, ?, NULL)
            ON CONFLICT(library_entry_id) DO UPDATE SET status = 'running', items_done = 0, items_total = NULL, error_message = NULL, started_at = excluded.started_at
            "#,
            vec![
                Value::Int(Some(library_entry_id)),
                Value::String(Some(Box::new(now))),
            ],
        ))
        .await?;
        Ok(())
    }

    pub async fn update_progress(
        conn: &DatabaseConnection,
        library_entry_id: i32,
        items_done: i32,
        items_total: Option<i32>,
    ) -> Result<(), DbErr> {
        conn.execute(Statement::from_sql_and_values(
            sea_orm::DbBackend::Sqlite,
            "UPDATE library_entry_sync_status SET items_done = ?, items_total = ? WHERE library_entry_id = ?",
            vec![
                Value::Int(Some(items_done)),
                items_total.map(|v| Value::Int(Some(v))).unwrap_or(Value::Int(None)),
                Value::Int(Some(library_entry_id)),
            ],
        ))
        .await?;
        Ok(())
    }

    pub async fn set_done(conn: &DatabaseConnection, library_entry_id: i32) -> Result<(), DbErr> {
        let now = Utc::now().to_rfc3339();
        conn.execute(Statement::from_sql_and_values(
            sea_orm::DbBackend::Sqlite,
            "UPDATE library_entry_sync_status SET status = 'done', error_message = NULL, synced_at = ? WHERE library_entry_id = ?",
            vec![
                Value::String(Some(Box::new(now))),
                Value::Int(Some(library_entry_id)),
            ],
        ))
        .await?;
        Ok(())
    }

    pub async fn set_error(
        conn: &DatabaseConnection,
        library_entry_id: i32,
        message: &str,
    ) -> Result<(), DbErr> {
        conn.execute(Statement::from_sql_and_values(
            sea_orm::DbBackend::Sqlite,
            "UPDATE library_entry_sync_status SET status = 'error', error_message = ? WHERE library_entry_id = ?",
            vec![
                Value::String(Some(Box::new(message.to_string()))),
                Value::Int(Some(library_entry_id)),
            ],
        ))
        .await?;
        Ok(())
    }

    /// Returns sync statuses for all direct children of `parent_id` that have a status row.
    pub async fn get_for_parent(
        conn: &DatabaseConnection,
        parent_id: i32,
    ) -> Result<Vec<SyncStatus>, DbErr> {
        let rows = SyncStatusRow::find_by_statement(Statement::from_sql_and_values(
            sea_orm::DbBackend::Sqlite,
            r#"
            SELECT ss.library_entry_id, ss.status, ss.items_done, ss.items_total,
                   ss.error_message, ss.started_at, ss.synced_at
            FROM library_entry_sync_status ss
            JOIN library_entry le ON le.id = ss.library_entry_id
            WHERE le.parent_id = ?
            "#,
            vec![Value::Int(Some(parent_id))],
        ))
        .all(conn)
        .await?;

        Ok(rows.into_iter().map(SyncStatusRow::into_model).collect())
    }
}


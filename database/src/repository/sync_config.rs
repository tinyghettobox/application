use chrono::Utc;
use sea_orm::{
    ConnectionTrait, DatabaseConnection, DbErr, FromQueryResult, Statement, Value,
};

use crate::model::sync_config::{CreateSyncConfig, SortBy, SyncConfig};

pub struct SyncConfigRepository;

impl SyncConfigRepository {
    pub async fn upsert(
        conn: &DatabaseConnection,
        library_entry_id: i32,
        config: CreateSyncConfig,
    ) -> Result<SyncConfig, DbErr> {
        conn.execute(Statement::from_sql_and_values(
            sea_orm::DbBackend::Sqlite,
            r#"
            INSERT INTO library_entry_sync_config
                (library_entry_id, sort_by, sort_regex, sync_interval_days)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(library_entry_id) DO UPDATE SET
                sort_by = excluded.sort_by,
                sort_regex = excluded.sort_regex,
                sync_interval_days = excluded.sync_interval_days
            "#,
            vec![
                Value::Int(Some(library_entry_id)),
                Value::String(Some(Box::new(config.sort_by.as_str().to_string()))),
                config.sort_regex.clone().map(|s| Value::String(Some(Box::new(s)))).unwrap_or(Value::String(None)),
                Value::Int(Some(config.sync_interval_days)),
            ],
        ))
        .await?;

        Self::get(conn, library_entry_id)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound("sync_config not found after upsert".into()))
    }

    pub async fn get(
        conn: &DatabaseConnection,
        library_entry_id: i32,
    ) -> Result<Option<SyncConfig>, DbErr> {
        #[derive(FromQueryResult)]
        struct Row {
            library_entry_id: i32,
            sort_by: String,
            sort_regex: Option<String>,
            sync_interval_days: i32,
            last_synced_at: Option<String>,
        }

        let row = Row::find_by_statement(Statement::from_sql_and_values(
            sea_orm::DbBackend::Sqlite,
            "SELECT library_entry_id, sort_by, sort_regex, sync_interval_days, last_synced_at FROM library_entry_sync_config WHERE library_entry_id = ?",
            vec![Value::Int(Some(library_entry_id))],
        ))
        .one(conn)
        .await?;

        Ok(row.map(|r| SyncConfig {
            library_entry_id: r.library_entry_id,
            sort_by: SortBy::from_str(&r.sort_by),
            sort_regex: r.sort_regex,
            sync_interval_days: r.sync_interval_days,
            last_synced_at: r.last_synced_at.and_then(|s| {
                chrono::DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc))
            }),
        }))
    }

    /// Returns entries that are due for sync: either never synced or past their interval.
    /// Returns (library_entry_id, spotify_id, spotify_type, name, sync_config)
    pub async fn find_next_due(
        conn: &DatabaseConnection,
    ) -> Result<Option<(i32, String, String, String, SyncConfig)>, DbErr> {
        #[derive(FromQueryResult)]
        struct Row {
            library_entry_id: i32,
            spotify_id: String,
            spotify_type: String,
            name: String,
            sort_by: String,
            sort_regex: Option<String>,
            sync_interval_days: i32,
            last_synced_at: Option<String>,
        }

        let now = Utc::now().to_rfc3339();

        let row = Row::find_by_statement(Statement::from_sql_and_values(
            sea_orm::DbBackend::Sqlite,
            r#"
            SELECT
                sc.library_entry_id,
                ts.spotify_id,
                ts.spotify_type,
                le.name,
                sc.sort_by,
                sc.sort_regex,
                sc.sync_interval_days,
                sc.last_synced_at
            FROM library_entry_sync_config sc
            JOIN track_source ts ON ts.library_entry_id = sc.library_entry_id
            JOIN library_entry le ON le.id = sc.library_entry_id
            LEFT JOIN library_entry_sync_status ss ON ss.library_entry_id = sc.library_entry_id
            WHERE
                (ss.status IS NULL OR ss.status NOT IN ('running'))
                AND sc.sync_interval_days > 0
                AND (
                    sc.last_synced_at IS NULL
                    OR datetime(sc.last_synced_at, '+' || sc.sync_interval_days || ' days') <= datetime(?)
                )
            ORDER BY sc.last_synced_at ASC NULLS FIRST, sc.library_entry_id DESC
            LIMIT 1
            "#,
            vec![Value::String(Some(Box::new(now)))],
        ))
        .one(conn)
        .await?;

        Ok(row.map(|r| {
            let config = SyncConfig {
                library_entry_id: r.library_entry_id,
                sort_by: SortBy::from_str(&r.sort_by),
                sort_regex: r.sort_regex,
                sync_interval_days: r.sync_interval_days,
                last_synced_at: r.last_synced_at.and_then(|s| {
                    chrono::DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc))
                }),
            };
            (r.library_entry_id, r.spotify_id, r.spotify_type, r.name, config)
        }))
    }

    pub async fn update_last_synced_at(
        conn: &DatabaseConnection,
        library_entry_id: i32,
    ) -> Result<(), DbErr> {
        let now = Utc::now().to_rfc3339();
        conn.execute(Statement::from_sql_and_values(
            sea_orm::DbBackend::Sqlite,
            "UPDATE library_entry_sync_config SET last_synced_at = ? WHERE library_entry_id = ?",
            vec![
                Value::String(Some(Box::new(now))),
                Value::Int(Some(library_entry_id)),
            ],
        ))
        .await?;
        Ok(())
    }

    /// Get all library entry IDs that have a sync config (for migration of existing entries).
    pub async fn get_all_ids(conn: &DatabaseConnection) -> Result<Vec<i32>, DbErr> {
        #[derive(FromQueryResult)]
        struct Row {
            library_entry_id: i32,
        }
        let rows = Row::find_by_statement(Statement::from_sql_and_values(
            sea_orm::DbBackend::Sqlite,
            "SELECT library_entry_id FROM library_entry_sync_config",
            vec![],
        ))
        .all(conn)
        .await?;
        Ok(rows.into_iter().map(|r| r.library_entry_id).collect())
    }
}

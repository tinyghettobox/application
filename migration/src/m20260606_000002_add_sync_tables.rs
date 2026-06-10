use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        conn.execute_unprepared(
            r#"
            CREATE TABLE IF NOT EXISTS library_entry_sync_config (
                library_entry_id INTEGER PRIMARY KEY,
                sort_by          TEXT NOT NULL DEFAULT 'release_date_desc',
                sort_regex       TEXT,
                sync_interval_days INTEGER NOT NULL DEFAULT 30,
                last_synced_at   DATETIME,
                FOREIGN KEY (library_entry_id) REFERENCES library_entry(id) ON DELETE CASCADE
            )
            "#,
        )
        .await?;

        conn.execute_unprepared(
            r#"
            CREATE TABLE IF NOT EXISTS library_entry_sync_status (
                library_entry_id INTEGER PRIMARY KEY,
                status           TEXT NOT NULL DEFAULT 'pending',
                items_done       INTEGER NOT NULL DEFAULT 0,
                items_total      INTEGER,
                error_message    TEXT,
                started_at       DATETIME,
                synced_at        DATETIME,
                FOREIGN KEY (library_entry_id) REFERENCES library_entry(id) ON DELETE CASCADE
            )
            "#,
        )
        .await?;

        conn.execute_unprepared(
            r#"
            INSERT INTO library_entry_sync_config (library_entry_id, sort_by, sync_interval_days)
            SELECT ts.library_entry_id, 'release_date_asc', 30
            FROM track_source ts
            WHERE ts.spotify_type IN ('artist', 'album', 'playlist', 'show')
              AND ts.library_entry_id IS NOT NULL
              AND NOT EXISTS (
                  SELECT 1 FROM library_entry_sync_config sc
                  WHERE sc.library_entry_id = ts.library_entry_id
              )
            "#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();
        conn.execute_unprepared("DROP TABLE IF EXISTS library_entry_sync_status").await?;
        conn.execute_unprepared("DROP TABLE IF EXISTS library_entry_sync_config").await?;
        Ok(())
    }
}

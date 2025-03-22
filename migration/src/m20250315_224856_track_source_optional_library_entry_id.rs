use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        conn.execute_unprepared("ALTER TABLE track_source RENAME TO old_track_source")
            .await?;
        conn.execute_unprepared(
        r#"
                CREATE TABLE track_source (
                    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
                    library_entry_id INTEGER DEFAULT NULL,
                    title TEXT NOT NULL,
                    url TEXT,
                    spotify_id TEXT,
                    spotify_type TEXT,
                    file BLOB,
                    FOREIGN KEY (library_entry_id) REFERENCES library_entry (id) ON DELETE CASCADE ON UPDATE CASCADE
                )
            "#
        ).await?;
        conn.execute_unprepared("INSERT INTO track_source SELECT * FROM old_track_source")
            .await?;
        conn.execute_unprepared("DROP TABLE old_track_source")
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        conn.execute_unprepared("ALTER TABLE track_source RENAME TO new_track_source")
            .await?;
        conn.execute_unprepared(
            r#"
                CREATE TABLE track_source (
                    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
                    library_entry_id INTEGER NOT NULL,
                    title TEXT NOT NULL,
                    url TEXT,
                    spotify_id TEXT,
                    spotify_type TEXT,
                    file BLOB,
                    FOREIGN KEY (library_entry_id) REFERENCES library_entry (id) ON DELETE CASCADE ON UPDATE CASCADE
                )
            "#
        ).await?;
        conn.execute_unprepared("INSERT INTO track_source SELECT * FROM new_track_source")
            .await?;
        conn.execute_unprepared("DROP TABLE new_track_source")
            .await?;

        Ok(())
    }
}

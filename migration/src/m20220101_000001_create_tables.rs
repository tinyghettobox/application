use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let connection = manager.get_connection();

        connection
            .execute_unprepared(
                r#"
                    CREATE TABLE system_config (
                        id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
                        sleep_timer INTEGER NOT NULL,
                        idle_shutdown_timer INTEGER NOT NULL,
                        display_off_timer INTEGER NOT NULL,
                        hostname TEXT NOT NULL,
                        cpu_governor TEXT NOT NULL,
                        overclock_sd_card BOOLEAN NOT NULL,
                        log_to_ram BOOLEAN NOT NULL,
                        wait_for_network BOOLEAN NOT NULL,
                        initial_turbo BOOLEAN NOT NULL,
                        swap_enabled BOOLEAN NOT NULL,
                        hdmi_rotate INTEGER NOT NULL,
                        lcd_rotate INTEGER NOT NULL,
                        display_brightness INTEGER NOT NULL,
                        display_resolution_x INTEGER NOT NULL,
                        display_resolution_y INTEGER NOT NULL,
                        audio_device TEXT NOT NULL,
                        volume INTEGER NOT NULL,
                        max_volume INTEGER NOT NULL,
                        led_on_off_shim_pin INTEGER NOT NULL,
                        led_brightness INTEGER NOT NULL,
                        led_brightness_dimmed INTEGER NOT NULL,
                        power_off_btn_delay INTEGER NOT NULL
                    )
                "#,
            )
            .await?;

        connection
            .execute_unprepared(
                r#"
                    CREATE TABLE spotify_config (
                        id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
                        client_id TEXT NOT NULL,
                        secret_key TEXT NOT NULL,
                        refresh_token TEXT,
                        access_token TEXT,
                        expired_at TEXT,
                        username TEXT,
                        password TEXT
                    )
                "#,
            )
            .await?;

        connection
            .execute_unprepared(
                r#"
                    CREATE TABLE library_entry (
                        id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
                        parent_id INTEGER,
                        variant TEXT NOT NULL,
                        name TEXT NOT NULL,
                        image BLOB,
                        played_at TEXT,
                        FOREIGN KEY (parent_id) REFERENCES library_entry (id) ON DELETE CASCADE ON UPDATE CASCADE
                    )
                "#,
            )
            .await?;

        connection
            .execute_unprepared(
                r#"
                    CREATE TABLE track_source (
                        id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
                        library_entry_id INTEGER NOT NULL,
                        title TEXT NOT NULL,
                        url TEXT,
                        file BLOB,
                        spotify_id TEXT,
                        spotify_type TEXT,
                        FOREIGN KEY (library_entry_id) REFERENCES library_entry (id) ON DELETE CASCADE ON UPDATE CASCADE
                    )
                "#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let connection = manager.get_connection();

        connection.execute_unprepared(r#"DROP TABLE system_config"#).await?;
        connection.execute_unprepared(r#"DROP TABLE spotify_config"#).await?;
        connection.execute_unprepared(r#"DROP TABLE library_entry"#).await?;
        connection.execute_unprepared(r#"DROP TABLE track_source"#).await?;

        Ok(())
    }
}

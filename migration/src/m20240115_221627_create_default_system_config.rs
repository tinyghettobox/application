use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        conn.execute_unprepared(r#"
            INSERT INTO system_config (
                id,
                sleep_timer,
                idle_shutdown_timer,
                display_off_timer,
                hostname,
                cpu_governor,
                overclock_sd_card,
                log_to_ram,
                wait_for_network,
                initial_turbo,
                swap_enabled,
                hdmi_rotate,
                lcd_rotate,
                display_brightness,
                display_resolution_x,
                display_resolution_y,
                audio_device,
                volume,
                max_volume,
                led_on_off_shim_pin,
                led_brightness,
                led_brightness_dimmed,
                power_off_btn_delay
            )
            VALUES (1, 60, 5, 2, 'tinyghettobox', 'schedutil', false, true, false, true, true, 0, 0, 100, 800, 480, 'hifiberry-dac', 50, 100, 0, 100, 10, 2);
        "#).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();
        conn.execute_unprepared("DELETE FROM system_config WHERE id = 1").await?;

        Ok(())
    }
}

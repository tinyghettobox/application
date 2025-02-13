use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();
        conn.execute_unprepared("ALTER TABLE system_config ADD COLUMN power_off_pin INT DEFAULT 0").await?;
        conn.execute_unprepared("ALTER TABLE system_config ADD COLUMN cut_pin INT DEFAULT 0").await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();
        conn.execute_unprepared("ALTER TABLE library_entry DROP COLUMN power_off_pin").await?;
        conn.execute_unprepared("ALTER TABLE library_entry DROP COLUMN cut_pin").await?;

        Ok(())
    }
}

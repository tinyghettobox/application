use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();

        conn.execute_unprepared(r#"
            INSERT INTO spotify_config (
                id,
                client_id,
                secret_key,
                refresh_token,
                access_token,
                expired_at,
                username,
                password
            )
            VALUES (1, '', '', NULL, NULL, NULL, NULL, NULL);
        "#).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();
        conn.execute_unprepared("DELETE FROM system_config WHERE id = 1").await?;

        Ok(())
    }
}

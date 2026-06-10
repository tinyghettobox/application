use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

fn generate_ap_password() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::{SystemTime, UNIX_EPOCH};

    // Simple deterministic-enough random for a one-time generated AP password.
    // Uses system time + process id mixed with a hasher to produce 8 lowercase alphanum chars.
    let mut hasher = DefaultHasher::new();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    let pid = std::process::id();
    nanos.hash(&mut hasher);
    pid.hash(&mut hasher);
    let seed = hasher.finish();

    let charset: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut result = String::with_capacity(8);
    let mut val = seed;
    for _ in 0..8 {
        result.push(charset[(val % charset.len() as u64) as usize] as char);
        val = val.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    }
    result
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();
        conn.execute_unprepared(
            "ALTER TABLE system_config ADD COLUMN ap_password TEXT NOT NULL DEFAULT ''",
        )
        .await?;

        let password = generate_ap_password();
        conn.execute_unprepared(&format!(
            "UPDATE system_config SET ap_password = '{}' WHERE id = 1",
            password
        ))
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();
        conn.execute_unprepared("ALTER TABLE system_config DROP COLUMN ap_password")
            .await?;
        Ok(())
    }
}

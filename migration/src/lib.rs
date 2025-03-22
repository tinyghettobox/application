pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_tables;
mod m20240115_221627_create_default_spotify_config;
mod m20240115_221627_create_default_system_config;
mod m20240212_225127_create_root_library_entry;
mod m20240321_123652_add_library_entry_sort_key;
mod m20250129_230144_add_on_off_shim_pins;
mod m20250315_224856_track_source_optional_library_entry_id;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_tables::Migration),
            Box::new(m20240115_221627_create_default_system_config::Migration),
            Box::new(m20240115_221627_create_default_spotify_config::Migration),
            Box::new(m20240212_225127_create_root_library_entry::Migration),
            Box::new(m20240321_123652_add_library_entry_sort_key::Migration),
            Box::new(m20250129_230144_add_on_off_shim_pins::Migration),
            Box::new(m20250315_224856_track_source_optional_library_entry_id::Migration),
        ]
    }
}

use sea_orm::ConnectOptions;
pub use sea_orm::{Database, DatabaseConnection, DbErr};
use tracing::info;
use tracing::log::LevelFilter;

pub use migration::{Migrator, MigratorTrait};
pub use repository::library_entry::LibraryEntryRepository;
pub use repository::spotify_config::SpotifyConfigRepository;
pub use repository::system_config::SystemConfigRepository;
pub use repository::track_source::TrackSourceRepository;

pub mod model;
mod repository;
mod util;

fn get_connection_uri() -> String {
    "sqlite://tinyghettobox.sqlite?mode=rwc".to_owned()
}

pub async fn connect() -> Result<DatabaseConnection, DbErr> {
    let mut connect_options = ConnectOptions::new(get_connection_uri());
    connect_options.sqlx_logging_level(LevelFilter::Trace);

    let connection = Database::connect(connect_options).await?;
    info!("Connected to database");

    let migrations = Migrator::get_pending_migrations(&connection).await.unwrap();
    info!("Applying {:?} migrations", migrations.len());
    Migrator::up(&connection, None).await.unwrap();
    info!("Migrations installed");

    Ok(connection)
}

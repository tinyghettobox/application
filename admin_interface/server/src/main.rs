use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use std::sync::Arc;
use tokio::sync::Notify;
use tracing::info;
use tracing::level_filters::LevelFilter;

use crate::routes::*;
use database::connect;

mod configure_commands;
mod problem;
mod routes;
mod spotify_client;
mod sync_job;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .with_ansi(cfg!(not(target_arch = "aarch64")))
        .init();
    let port = std::env::var("PORT")
        .unwrap_or("8080".to_owned())
        .parse::<u16>()
        .unwrap();
    info!("Starting server on http://localhost:{} ...", port);

    let connection = connect().await.expect("Failed to connect to database");

    // Shared notify handle: route handlers signal it when a sync is triggered
    // so the background job wakes up immediately instead of waiting 5 minutes.
    let sync_notify = Arc::new(Notify::new());

    // Shared wifi connection state (polled by the frontend during setup).
    let wifi_state = routes::wifi::new_shared_wifi_state();

    // Start the background Spotify sync job.
    sync_job::start(connection.clone(), Arc::clone(&sync_notify));

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new("%r => %s sent %b bytes in %Dms"))
            .service(system_config::get)
            .service(system_config::update)
            .service(spotify::get_config)
            .service(spotify::update_config)
            .service(spotify::auth)
            .service(spotify::callback)
            .service(spotify::search)
            .service(spotify::children)
            .service(library::get)
            .service(library::delete)
            .service(library::create)
            .service(library::update)
            .service(library::upload)
            .service(library::bulk_update)
            .service(library::mark_played)
            .service(library::get_sync_config)
            .service(library::upsert_sync_config)
            .service(library::get_sync_status)
            .service(library::get_children_sync_statuses)
            .service(library::trigger_sync)
            .service(update::get_operating_system_update_version)
            .service(wifi::connect)
            .service(wifi::status)
            .service(image::proxy_image)
            .service(static_files::get)
            .app_data(web::Data::new(connection.clone()))
            .app_data(web::Data::new(Arc::clone(&sync_notify)))
            .app_data(web::Data::new(wifi_state.clone()))
            .app_data(web::JsonConfig::default().limit(100 * 1024 * 1024))
    })
    .bind(("0.0.0.0", port))
    .expect("Failed to bind to port")
    .run()
    .await
    .expect("Failed to run server");
}

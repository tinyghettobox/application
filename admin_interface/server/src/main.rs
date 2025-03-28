use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use tracing::info;
use tracing::level_filters::LevelFilter;

use crate::routes::*;
use database::connect;

mod commands;
mod error;
mod routes;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .init();
    let port = std::env::var("PORT")
        .unwrap_or("8080".to_owned())
        .parse::<u16>()
        .unwrap();
    info!("Starting server on http://localhost:{} ...", port);

    let connection = connect().await.expect("Failed to connect to database");

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
            .service(image::proxy_image)
            .service(static_files::get)
            .app_data(web::Data::new(connection.clone()))
            .app_data(web::JsonConfig::default().limit(100 * 1024 * 1024))
    })
    .bind(("0.0.0.0", port))
    .expect("Failed to bind to port")
    .run()
    .await
    .expect("Failed to run server");
}

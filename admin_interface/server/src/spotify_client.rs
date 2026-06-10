use actix_web::error::{ErrorBadRequest, ErrorInternalServerError};
use database::{DatabaseConnection, SpotifyConfigRepository};
use rspotify::{clients::BaseClient, AuthCodeSpotify, Credentials, Token};
use tracing::warn;

pub async fn get_spotify(conn: &DatabaseConnection) -> Result<AuthCodeSpotify, actix_web::Error> {
    let config = SpotifyConfigRepository::get(conn)
        .await
        .map_err(|e| ErrorBadRequest(e))?;

    if config.access_token.is_none() {
        return Err(ErrorBadRequest("Spotify is not configured"));
    }

    let token = match config.expired_at {
        Some(expired_at) => {
            let date = chrono::DateTime::parse_from_rfc3339(&expired_at)
                .map_err(|e| {
                    ErrorInternalServerError(format!(
                        "spotify_config.expired_at '{}' has invalid date format: {}",
                        expired_at, e
                    ))
                })?
                .to_utc();

            Token {
                access_token: config.access_token.unwrap(),
                refresh_token: config.refresh_token,
                expires_at: Some(date),
                expires_in: chrono::Utc::now() - date,
                scopes: Default::default(),
            }
        }
        None => return Err(ErrorBadRequest("No expired_at in spotify_config")),
    };

    let spotify = AuthCodeSpotify::from_token_with_config(
        token,
        Credentials::new(&config.client_id, &config.secret_key),
        Default::default(),
        Default::default(),
    );

    if let Err(error) = spotify.refresh_token() {
        warn!("Could not refresh spotify access token: {}", error);
    }

    Ok(spotify)
}

use actix_web::error::{ErrorBadRequest, ErrorInternalServerError};
use actix_web::{get, put, web, HttpRequest, HttpResponse, Responder, Result};
use database::{DatabaseConnection, SpotifyConfigRepository};
use rspotify::model::{AlbumId, ArtistId, PlaylistId, SearchType, ShowId};
use rspotify::prelude::{BaseClient, OAuthClient};
use rspotify::{AuthCodeSpotify, Credentials, OAuth, Token};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashSet;
use tracing::{error, warn};

#[get("/api/spotify/config")]
pub async fn get_config(conn: web::Data<DatabaseConnection>) -> impl Responder {
    match SpotifyConfigRepository::get(&conn).await {
        Ok(model) => HttpResponse::Ok().json(model),
        Err(error) => {
            error!("Failed to get system config: {:?}", error);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[put("/api/spotify/config")]
pub async fn update_config(conn: web::Data<DatabaseConnection>, json: web::Json<serde_json::Value>) -> actix_web::Result<impl Responder> {
    match SpotifyConfigRepository::update_from_json(&conn, json.into_inner()).await {
        Ok((updated_model, changed_fields)) => {
            if changed_fields.contains(&"username".to_string()) {
                crate::commands::set_spotifyd_config("username", &updated_model.username.clone().unwrap())?;
            }
            if changed_fields.contains(&"password".to_string()) {
                crate::commands::set_spotifyd_config("password", &updated_model.password.clone().unwrap())?;
            }
            Ok(HttpResponse::Ok().json(updated_model))
        },
        Err(error) => {
            error!("Failed to set system config: {:?}", error);
            Ok(HttpResponse::BadRequest().json(json!({ "error": error.to_string() })))
        }
    }
}

fn get_origin_host<'a>(http_request: HttpRequest) -> String {
    http_request
        .headers()
        .get("x-forwarded-host")
        .or(http_request.headers().get("host"))
        .and_then(|value| value.to_str().ok())
        .unwrap_or("localhost")
        .to_string()
}

#[get("/api/spotify/auth")]
pub async fn auth(conn: web::Data<DatabaseConnection>, request: HttpRequest) -> Result<HttpResponse> {
    let host = get_origin_host(request);
    let config = SpotifyConfigRepository::get(&conn).await.map_err(|e| ErrorBadRequest(e))?;

    let spotify = AuthCodeSpotify::new(
        Credentials::new(&config.client_id, &config.secret_key),
        OAuth {
            redirect_uri: format!("http://{}/api/spotify/auth/callback", host),
            scopes: HashSet::from([
                "streaming".to_string(),
                "user-read-currently-playing".to_string(),
                "user-read-playback-state".to_string(),
                "user-modify-playback-state".to_string(),
            ]),
            ..Default::default()
        },
    );

    Ok(HttpResponse::PermanentRedirect()
        .insert_header((
            "location",
            spotify.get_authorize_url(false).map_err(|e| ErrorBadRequest(e))?,
        ))
        .finish())
}

#[derive(Deserialize)]
pub struct CallbackParams {
    pub code: String,
    pub state: String,
}

#[get("/api/spotify/auth/callback")]
pub async fn callback(
    conn: web::Data<DatabaseConnection>,
    query: web::Query<CallbackParams>,
    request: HttpRequest,
) -> Result<HttpResponse>
{
    let host = get_origin_host(request);
    let mut config = SpotifyConfigRepository::get(&conn).await.map_err(|e| ErrorBadRequest(e))?;

    let spotify = AuthCodeSpotify::new(
        Credentials::new(&config.client_id, &config.secret_key),
        OAuth {
            redirect_uri: format!("http://{}/api/spotify/auth/callback", host),
            state: query.state.clone(),
            ..Default::default()
        },
    );

    spotify.request_token(&query.code).map_err(|e| ErrorBadRequest(e))?;
    let token = spotify.get_token().lock().unwrap().clone().unwrap();

    config.access_token = Some(token.access_token);
    config.refresh_token = token.refresh_token;
    config.expired_at = token.expires_at.map(|date| date.to_rfc3339());

    SpotifyConfigRepository::update(&conn, config).await.map_err(|e| ErrorInternalServerError(e))?;

    Ok(HttpResponse::PermanentRedirect().insert_header(("location", "/spotifyConfig/2")).finish())
}

#[derive(Deserialize)]
pub struct SearchPayload {
    pub search: String,
    pub search_type: SearchType,
}

#[get("/api/spotify/search")]
pub async fn search(conn: web::Data<DatabaseConnection>, params: web::Query<SearchPayload>) -> Result<HttpResponse> {
    let spotify = get_spotify(&conn).await?;
    let result = spotify
        .search(&params.search, params.search_type, None, None, Some(50), None)
        .map_err(|e| ErrorBadRequest(format!("Spotify search failed: {}", e)))?;

    Ok(HttpResponse::Ok().json(result))
}

#[derive(Deserialize)]
pub struct ChildrenParams {
    pub parent_type: String,
    pub parent_id: String,
    pub offset: Option<u32>,
}

#[get("/api/spotify/children")]
pub async fn children(
    conn: web::Data<DatabaseConnection>,
    params: web::Query<ChildrenParams>,
) -> Result<HttpResponse> {
    let spotify = get_spotify(&conn).await?;

    Ok(match params.parent_type.as_str() {
        "artist" => HttpResponse::Ok().json(
            spotify
                .artist_albums_manual(
                    ArtistId::from_id(params.parent_id.as_str())
                        .map_err(|e| ErrorInternalServerError(format!("Invalid spotify id '{}': {}", params.parent_id, e)))?,
                    vec![],
                    None,
                    Some(50),
                    params.offset,
                )
                .map_err(|e| ErrorBadRequest(e))?,
        ),
        "album" => HttpResponse::Ok().json(
            spotify
                .album_track_manual(
                    AlbumId::from_id(params.parent_id.as_str())
                        .map_err(|e| ErrorInternalServerError(format!("Invalid spotify id '{}': {}", params.parent_id, e)))?,
                    None,
                    Some(50),
                    params.offset,
                )
                .map_err(|e| ErrorBadRequest(e))?,
        ),
        "playlist" => HttpResponse::Ok().json(
            spotify
                .playlist_items_manual(
                    PlaylistId::from_id(params.parent_id.as_str())
                        .map_err(|e| ErrorInternalServerError(format!("Invalid spotify id '{}': {}", params.parent_id, e)))?,
                    None,
                    None,
                    Some(50),
                    params.offset,
                )
                .map_err(|e| ErrorBadRequest(e))?,
        ),
        "show" => HttpResponse::Ok().json(
            spotify
                .get_shows_episodes_manual(
                    ShowId::from_id(params.parent_id.as_str())
                        .map_err(|e| ErrorInternalServerError(format!("Invalid spotify id '{}': {}", params.parent_id, e)))?,
                    None,
                    Some(50),
                    params.offset,
                )
                .map_err(|e| ErrorBadRequest(e))?,
        ),
        _ => {
            return Err(ErrorInternalServerError(format!(
                "Unknown spotify uri type {}:{}",
                params.parent_type, params.parent_id
            )))
        }
    })
}

async fn get_spotify(conn: &DatabaseConnection) -> Result<AuthCodeSpotify, actix_web::Error> {
    let config = SpotifyConfigRepository::get(conn).await.map_err(|e| ErrorBadRequest(e))?;

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
            //
            // if date < chrono::Utc::now() {
            //     return Err(ErrorBadRequest("Spotify token is expired"));
            // }

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
        Default::default()
    );

    if let Err(error) = spotify.refresh_token() {
        warn!("Could not refresh spotify access token: {}", error);
    }

    Ok(spotify)
}

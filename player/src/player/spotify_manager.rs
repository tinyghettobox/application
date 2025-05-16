use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use chrono::{DateTime, Utc};
use rspotify::clients::BaseClient;
use rspotify::{AuthCodeSpotify, Credentials, Token};
use serde_json::json;
use std::collections::HashSet;
use std::fs::{create_dir_all, write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};

use database::{DatabaseConnection, SpotifyConfigRepository};

#[derive(Clone)]
pub struct SpotifyManager {
    pub client: AuthCodeSpotify,
    conn: DatabaseConnection,
    spotifyd_initialized: Arc<AtomicBool>,
}

impl SpotifyManager {
    pub async fn new(conn: &DatabaseConnection) -> Self {
        let config = SpotifyConfigRepository::get(conn)
            .await
            .expect("Could not get spotify config");
        let expired_at = config.expired_at.map(|date| {
            date.parse::<DateTime<Utc>>()
                .expect("Expired at should be a valid date")
        });
        let token = Token {
            access_token: config.access_token.to_owned().unwrap_or("".to_owned()),
            refresh_token: config.refresh_token.to_owned(),
            expires_at: expired_at,
            expires_in: Utc::now() - expired_at.unwrap_or_default(),
            scopes: HashSet::from(
                [
                    "streaming",
                    "user-read-currently-playing",
                    "user-modify-playback-state",
                    "user-read-playback-state",
                    "user-read-private",
                    "user-read-email",
                ]
                .map(|s| s.to_string()),
            ),
        };

        let client = AuthCodeSpotify::from_token_with_config(
            token.clone(),
            Credentials {
                id: config.client_id.to_owned(),
                secret: Some(config.secret_key.to_owned()),
            },
            Default::default(),
            Default::default(),
        );

        if let Err(error) = client.refresh_token() {
            error!("Could not refresh spotify access token: {}", error);
        }

        let spotify = Self {
            client,
            conn: conn.clone(),
            spotifyd_initialized: Arc::new(AtomicBool::new(false)),
        };

        spotify.start_polling_config();
        spotify.start_token_refresh();

        spotify
    }

    pub fn start_polling_config(&self) {
        let self_ = self.clone();
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(5)).await;
                {
                    let config = SpotifyConfigRepository::get(&self_.conn)
                        .await
                        .expect("Could not get spotify config");
                    let config_expired_at = config.expired_at.map(|date| {
                        date.parse::<DateTime<Utc>>()
                            .expect("Expired at should be a valid date")
                    });

                    let mut current_token = self_.client.get_token().lock().unwrap().clone();

                    if let Some(current_token) = current_token.as_ref() {
                        if current_token.access_token
                            == config.access_token.to_owned().unwrap_or("".to_string())
                        {
                            continue;
                        }
                        if current_token.expires_at == config_expired_at {
                            continue;
                        }
                    }
                    info!("Loading new spotify config");

                    let token = Token {
                        access_token: config.access_token.to_owned().unwrap_or("".to_owned()),
                        refresh_token: config.refresh_token.to_owned(),
                        expires_at: config_expired_at,
                        expires_in: Utc::now() - config_expired_at.unwrap_or_default(),
                        scopes: HashSet::from(
                            [
                                "streaming",
                                "user-read-currently-playing",
                                "user-modify-playback-state",
                                "user-read-playback-state",
                                "user-read-private",
                                "user-read-email",
                            ]
                            .map(|s| s.to_string()),
                        ),
                    };

                    current_token.replace(token);
                }
            }
        });
    }

    pub fn start_token_refresh(&self) {
        let mut self_ = self.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(15)).await;
            loop {
                {
                    info!("Refreshing spotify token");
                    if let Err(error) = self_.client.refresh_token() {
                        warn!("Could not refresh spotify access token: {}", error);
                    }
                    let new_token = match self_.client.get_token().lock() {
                        Ok(token) => token.clone(),
                        Err(error) => {
                            error!("Could not lock spotify token: {}", error);
                            continue;
                        }
                    };
                    let mut config = SpotifyConfigRepository::get(&self_.conn)
                        .await
                        .expect("Could not get spotify config");
                    if let Some(token) = new_token.as_ref() {
                        if !token.access_token.is_empty()
                            && token.access_token != config.access_token.unwrap_or("".to_string())
                        {
                            info!("Updating spotify access token");
                            config.access_token = Some(token.access_token.to_owned());
                            config.refresh_token = token.refresh_token.clone();
                            config.expired_at =
                                token.expires_at.clone().map(|date| date.to_rfc3339());
                            SpotifyConfigRepository::update(&self_.conn, config)
                                .await
                                .expect("Could not save spotify token");

                            if let Err(error) = self_.update_spotifyd_credentials(&token) {
                                warn!("Error updating spotify credentials {}", error);
                            }
                        } else {
                            info!("Access token did not yet change");
                        }
                    }
                }
                sleep(Duration::from_secs(360)).await;
            }
        });
    }

    fn update_spotifyd_credentials(&mut self, token: &Token) -> Result<(), String> {
        if cfg!(target_os = "linux") {
            info!("Updating spotifyd credentials");
            let credentials_dir = Path::new("/srv/spotifyd_cache/oauth");
            if !credentials_dir.exists() {
                create_dir_all(credentials_dir).expect("Could not create credentials dir");
            }

            let payload = json!({"username": "", "auth_type": 3, "auth_data": BASE64_STANDARD.encode(token.access_token.to_owned())});
            let content = serde_json::to_string(&payload)
                .map_err(|e| format!("Could not serialize credentials json: {}", e))?;

            write(credentials_dir.join("credentials.json"), content)
                .map_err(|e| format!("Could not save credentials file: {}", e))?;

            // On first start we update the spotifyd token and restart the service. Spotifyd does not
            // pick up changed credential file during run, so we have to restart to make it active.
            // Spotifyd does token rotation but if the token is once outdated without refresh token.
            // If the token is outdated once, it can't get new token, which is why we update it here

            if !self.spotifyd_initialized.load(Ordering::Acquire) {
                info!("Restarting spotifyd");
                let _ = std::process::Command::new("systemctl")
                    .arg("restart")
                    .arg("spotifyd")
                    .spawn();
                self.spotifyd_initialized.store(true, Ordering::Release);
            }
        }
        Ok(())
    }
}

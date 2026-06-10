use actix_web::{get, post, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tracing::{error, info, warn};

/// In-process wifi connection state.  Only one connection attempt at a time.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WifiConnectionStatus {
    Idle,
    Connecting,
    Connected,
    Failed,
}

#[derive(Debug, Clone, Serialize)]
pub struct WifiState {
    pub status: WifiConnectionStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Default for WifiState {
    fn default() -> Self {
        WifiState {
            status: WifiConnectionStatus::Idle,
            error: None,
        }
    }
}

pub type SharedWifiState = Arc<Mutex<WifiState>>;

pub fn new_shared_wifi_state() -> SharedWifiState {
    Arc::new(Mutex::new(WifiState::default()))
}

#[derive(Deserialize)]
pub struct ConnectRequest {
    pub ssid: String,
    pub password: String,
    pub security: String, // WPA | WEP | nopass
}

/// POST /api/wifi/connect
/// Starts a wifi connection attempt in the background.
/// Returns immediately with `{ "status": "connecting" }`.
/// Poll GET /api/wifi/status for the outcome.
#[post("/api/wifi/connect")]
pub async fn connect(
    body: web::Json<ConnectRequest>,
    wifi_state: web::Data<SharedWifiState>,
) -> impl Responder {
    {
        let mut state = wifi_state.lock().unwrap();
        state.status = WifiConnectionStatus::Connecting;
        state.error = None;
    }

    let ssid = body.ssid.clone();
    let password = body.password.clone();
    let security = body.security.clone();
    let wifi_state_bg = Arc::clone(&wifi_state);

    tokio::spawn(async move {
        info!("Attempting wifi connection to SSID '{}'", ssid);
        let output = tokio::process::Command::new("/srv/tinyghettobox/set_wifi.sh")
            .arg(&ssid)
            .arg(&password)
            .arg(&security)
            .output()
            .await;

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();

                let mut state = wifi_state_bg.lock().unwrap();
                if out.status.success() && stdout.starts_with("Ok") {
                    info!("Wifi connection succeeded");
                    state.status = WifiConnectionStatus::Connected;
                    state.error = None;
                } else {
                    let err = if !stderr.is_empty() { stderr } else { stdout };
                    warn!("Wifi connection failed: {}", err);
                    state.status = WifiConnectionStatus::Failed;
                    state.error = Some(err);
                }
            }
            Err(e) => {
                error!("Failed to spawn set_wifi.sh: {}", e);
                let mut state = wifi_state_bg.lock().unwrap();
                state.status = WifiConnectionStatus::Failed;
                state.error = Some(format!("Failed to run wifi script: {}", e));
            }
        }
    });

    HttpResponse::Accepted().json(WifiState {
        status: WifiConnectionStatus::Connecting,
        error: None,
    })
}

/// GET /api/wifi/status
/// Returns the current wifi connection state.
#[get("/api/wifi/status")]
pub async fn status(wifi_state: web::Data<SharedWifiState>) -> impl Responder {
    let state = wifi_state.lock().unwrap().clone();
    HttpResponse::Ok().json(state)
}

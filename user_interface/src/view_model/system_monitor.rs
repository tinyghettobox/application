use tokio::sync::oneshot;

use crate::model::{
    actions::{Action, AudioStatus, WifiStatus},
    State,
};

pub struct SystemMonitorVM;

impl SystemMonitorVM {
    pub fn new(state: State, audio_ready_tx: oneshot::Sender<()>) -> Self {
        {
            let state = state.clone();
            tokio::spawn(async move {
                monitor_audio(state, audio_ready_tx).await;
            });
        }
        {
            let state = state.clone();
            tokio::spawn(async move {
                monitor_wifi(state).await;
            });
        }
        SystemMonitorVM
    }
}

// ── Linux implementation ────────────────────────────────────────────────────

#[cfg(target_os = "linux")]
async fn monitor_audio(state: State, tx: oneshot::Sender<()>) {
    use std::time::{Duration, Instant};

    const POLL_INTERVAL: Duration = Duration::from_secs(2);
    const TIMEOUT: Duration = Duration::from_secs(30);

    let start = Instant::now();

    loop {
        if is_audio_ready() {
            tracing::info!("Audio system ready — PipeWire socket detected, initialising player");
            // Signal main.rs to proceed with player init
            let _ = tx.send(());
            state.dispatch(Action::SetAudioStatus(AudioStatus::Ready));
            return;
        }

        if start.elapsed() >= TIMEOUT {
            tracing::error!("Audio system never became available after 30 s; music playback is disabled");
            state.dispatch(Action::SetAudioStatus(AudioStatus::Failed));
            return;
        }

        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

/// Check whether the PipeWire socket is present, indicating the audio daemon
/// is up and accepting connections. This is faster and more reliable than
/// attempting to open a device (which blocks for several seconds on failure).
#[cfg(target_os = "linux")]
fn is_audio_ready() -> bool {
    // Prefer XDG_RUNTIME_DIR (set by the session manager, works for any UID).
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .unwrap_or_else(|_| format!("/run/user/{}", libc_getuid()));

    let pipewire_socket = format!("{}/pipewire-0", runtime_dir);
    if std::path::Path::new(&pipewire_socket).exists() {
        return true;
    }

    // Fallback: PulseAudio-compatible socket (some PipeWire setups expose this).
    let pulse_socket = format!("{}/pulse/native", runtime_dir);
    std::path::Path::new(&pulse_socket).exists()
}

/// Read the process UID without depending on the `libc` crate.
#[cfg(target_os = "linux")]
fn libc_getuid() -> u32 {
    // /proc/self/status always contains a `Uid:` line.
    std::fs::read_to_string("/proc/self/status")
        .unwrap_or_default()
        .lines()
        .find(|l| l.starts_with("Uid:"))
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000)
}

#[cfg(target_os = "linux")]
async fn monitor_wifi(state: State) {
    use std::time::Duration;

    // Initial dispatch so the UI shows "initializing" briefly and then
    // transitions to whatever the real state is.
    tokio::time::sleep(Duration::from_millis(500)).await;

    loop {
        let status = read_wifi_status();
        if matches!(status, WifiStatus::Weak(_)) {
            tracing::warn!("WiFi signal is weak — playback of network streams may be unstable");
        }
        state.dispatch(Action::SetWifiStatus(status));
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

/// Parse `/proc/net/wireless` to determine connectivity and signal strength.
///
/// File format (after two header lines):
/// ```text
///  wlan0: 0000   70.  -40.  -256.   0   0   0   0   0   0
/// ```
/// Columns: iface status link level noise …
#[cfg(target_os = "linux")]
fn read_wifi_status() -> WifiStatus {
    let content = match std::fs::read_to_string("/proc/net/wireless") {
        Ok(c) => c,
        Err(_) => return WifiStatus::Disconnected,
    };

    for line in content.lines().skip(2) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // The level column (index 3) is an i8 dBm value followed by a dot.
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }
        let level_str = parts[3].trim_end_matches('.');
        match level_str.parse::<i8>() {
            Ok(level) if level > -100 => {
                return if level > -55 {
                    WifiStatus::Strong(level)
                } else if level > -75 {
                    WifiStatus::Good(level)
                } else {
                    WifiStatus::Weak(level)
                };
            }
            _ => {
                // Interface present but level is implausible (−256 sentinel) →
                // associated but no signal, or not associated.
                return WifiStatus::Disconnected;
            }
        }
    }

    // No interfaces found in the file.
    WifiStatus::Disconnected
}

// ── Non-Linux stub (Windows dev environment) ────────────────────────────────

#[cfg(not(target_os = "linux"))]
async fn monitor_audio(state: State, tx: oneshot::Sender<()>) {
    tracing::debug!("SystemMonitor: non-Linux stub — audio immediately ready");
    let _ = tx.send(());
    state.dispatch(Action::SetAudioStatus(AudioStatus::Ready));
}

#[cfg(not(target_os = "linux"))]
async fn monitor_wifi(state: State) {
    tracing::debug!("SystemMonitor: non-Linux stub — wifi immediately strong");
    state.dispatch(Action::SetWifiStatus(WifiStatus::Strong(0)));
}

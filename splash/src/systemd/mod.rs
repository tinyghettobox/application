use std::sync::{Arc, Mutex};
use std::thread;
use zbus::{Connection, Result as ZbusResult};
use crate::state::SplashState;

/// Verbindet zur systemd D-Bus-Schnittstelle
async fn connect_to_systemd() -> ZbusResult<Connection> {
    Connection::system().await
}

/// Ruft die Liste aller systemd-Units ab
async fn get_systemd_units(conn: &Connection) -> ZbusResult<Vec<(String, String, String, String, String)>> {
    let proxy = zbus::Proxy::new(
        conn,
        "org.freedesktop.systemd1",
        "/org/freedesktop/systemd1",
        "org.freedesktop.systemd1.Manager",
    ).await?;

    let units: Vec<(String, String, String, String, String)> = proxy
        .call("ListUnits", &())
        .await?;

    Ok(units)
}

/// Prüft, ob ein bestimmtes Target aktiv ist
async fn is_target_active(conn: &Connection, target: &str) -> bool {
    let proxy = zbus::Proxy::new(
        conn,
        "org.freedesktop.systemd1",
        "/org/freedesktop/systemd1",
        "org.freedesktop.systemd1.Manager",
    ).await.ok();

    if let Some(proxy) = proxy {
        let unit_path: zbus::Result<String> = proxy
            .call("GetUnit", &[target])
            .await;

        if let Ok(unit_path) = unit_path {
            let unit_proxy = zbus::Proxy::new(
                conn,
                "org.freedesktop.systemd1",
                &unit_path,
                "org.freedesktop.systemd1.Unit",
            ).await.ok();

            if let Some(unit_proxy) = unit_proxy {
                let active_state: zbus::Result<String> = unit_proxy
                    .get_property("ActiveState")
                    .await;

                return matches!(active_state, Ok(state) if state == "active");
            }
        }
    }

    false
}

/// Startet einen Thread zur Überwachung des systemd-Boot-Fortschritts
pub fn spawn_systemd_monitor(state: Arc<Mutex<SplashState>>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to build tokio runtime");

        rt.block_on(async {
            // Verbindung zu systemd herstellen
            let conn = match connect_to_systemd().await {
                Ok(conn) => conn,
                Err(e) => {
                    eprintln!("Fehler beim Verbinden zu systemd: {}", e);
                    return;
                }
            };

            // Initiale Nachricht setzen
            if let Ok(mut state_guard) = state.lock() {
                state_guard.message = Some("System wird gestartet...".to_string());
                state_guard.progress = 0;
            }

            // Gesamtanzahl der Units ermitteln
            let total_units = match get_systemd_units(&conn).await {
                Ok(units) => units.len(),
                Err(_) => 100, // Fallback-Wert
            };

            let mut last_progress = 0;
            let mut finished = false;

            while !finished {
                // Aktuelle Units abrufen
                if let Ok(units) = get_systemd_units(&conn).await {
                    // Aktive und fehlgeschlagene Units zählen
                    let active_units = units.iter()
                        .filter(|u| u.3 == "active")
                        .count();
                    
                    let failed_units = units.iter()
                        .filter(|u| u.3 == "failed")
                        .count();

                    // Fortschritt berechnen
                    let current_units = active_units + failed_units;
                    let mut progress = (current_units * 100) / total_units;
                    if progress > 100 {
                        progress = 100;
                    }

                    // Prüfen, ob multi-user.target erreicht wurde
                    if is_target_active(&conn, "multi-user.target").await {
                        progress = 100;
                        finished = true;
                    }

                    // Zustand aktualisieren, wenn sich der Fortschritt geändert hat
                    if progress as u8 != last_progress {
                        if let Ok(mut state_guard) = state.lock() {
                            state_guard.progress = progress as u8;
                            
                            // Nachricht mit Fortschritt aktualisieren
                            if failed_units > 0 {
                                state_guard.message = Some(format!(
                                    "System wird gestartet... {}% ({} fehlgeschlagen)",
                                    progress, failed_units
                                ));
                            } else {
                                state_guard.message = Some(format!(
                                    "System wird gestartet... {}%",
                                    progress
                                ));
                            }

                            // Abschlussnachricht, wenn fertig
                            if progress >= 100 {
                                state_guard.message = Some("System bereit".to_string());
                            }
                        }
                        
                        last_progress = progress as u8;
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }

            // Warten nach Abschluss und dann beenden
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            std::process::exit(0);
        });
    })
}
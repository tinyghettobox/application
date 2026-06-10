#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use std::sync::mpsc::sync_channel;

use image::{GenericImageView, ImageReader};
use slint::{Image, Model, ModelRc, SharedPixelBuffer};
use tracing_subscriber::{filter::LevelFilter, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

slint::include_modules!();

mod log_layer;
mod model;
mod view_model;

fn main() -> Result<(), Box<dyn Error>> {
    // Channel bridges the tracing layer (built before State) to State::dispatch (available after).
    let (log_tx, log_rx) = sync_channel::<model::actions::Action>(256);

    tracing_subscriber::registry()
        .with(EnvFilter::builder()
            .with_default_directive(LevelFilter::DEBUG.into())
            .from_env_lossy())
        .with(tracing_subscriber::fmt::layer().with_ansi(cfg!(not(target_arch = "aarch64"))))
        .with(log_layer::StateLogLayer::new(log_tx))
        .init();

    // Tokio runtime on background threads; Slint event loop stays on the main thread.
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    // DB connect and State creation only — Player::new() is deferred until the
    // audio daemon (PipeWire) signals it is ready via the oneshot channel.
    // Also read setup_complete synchronously so we can show the setup screen
    // immediately without waiting for the async LoadSystemConfig round-trip.
    let (state, conn_for_player, initial_setup_complete) = rt.block_on(async {
        let conn = database::connect().await.expect("DB connect failed");
        let conn_for_player = conn.clone();
        let setup_complete = database::SystemConfigRepository::get(&conn)
            .await
            .ok()
            .flatten()
            .map(|c| c.setup_complete)
            .unwrap_or(false);
        let state = model::State::new(conn);
        (state, conn_for_player, setup_complete)
    });

    // Oneshot used by SystemMonitorVM to wake the player-init task.
    let (audio_ready_tx, audio_ready_rx) = tokio::sync::oneshot::channel::<()>();

    // Forward log actions from the tracing layer into State::dispatch.
    {
        let state_for_log = state.clone();
        std::thread::spawn(move || {
            for action in log_rx {
                state_for_log.dispatch(action);
            }
        });
    }

    // Enter the runtime so tokio::spawn works from the action loop
    let _guard = rt.enter();

    let ui = AppWindow::new()?;

    let _content_vm = view_model::content::ContentVM::new(ui.as_weak(), state.clone());
    let _display_timer_vm = view_model::display_timer::DisplayTimerVM::new(ui.as_weak(), state.clone());
    let _logs_vm = view_model::logs::LogsVM::new(ui.as_weak(), state.clone());
    let _messages_vm = view_model::messages::MessagesVM::new(ui.as_weak(), state.clone());
    let _navbar_vm = view_model::navbar::NavbarVM::new(ui.as_weak(), state.clone());
    let _playbar_vm = view_model::playbar::PlaybarVM::new(ui.as_weak(), state.clone());
    // SystemMonitorVM polls audio/wifi readiness and dispatches status actions.
    // On non-Linux it immediately signals Ready so behaviour is unchanged.
    let _system_monitor_vm = view_model::system_monitor::SystemMonitorVM::new(state.clone(), audio_ready_tx);
    let _setup_vm = view_model::setup::SetupVM::new(ui.as_weak(), state.clone(), initial_setup_complete);

    // Apply initial setup visibility synchronously before the first frame so
    // the correct screen is shown without any flicker.
    if !initial_setup_complete {
        ui.global::<Setup>().set_visible(true);
    }

    // Start loading library content immediately; if setup is not complete the
    // SetupView overlay will hide it until setup is done.
    state.dispatch(model::actions::Action::LoadLibraryEntry(0));

    // Load system config — SetupVM will keep setup visibility in sync and
    // dispatch LoadLibraryEntry(0) once setup_complete transitions to true.
    state.dispatch(model::actions::Action::LoadSystemConfig);
    {
        let state_for_poll = state.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(3));
            interval.tick().await; // skip the immediate first tick (already dispatched above)
            loop {
                interval.tick().await;
                state_for_poll.dispatch(model::actions::Action::LoadSystemConfig);
            }
        });
    }

    // Init the player once the system monitor confirms the audio daemon is up.
    {
        let state_clone = state.clone();
        tokio::spawn(async move {
            // Block until SystemMonitorVM fires the audio-ready signal (or drops on failure).
            if audio_ready_rx.await.is_err() {
                // Sender dropped without sending means AudioStatus::Failed was dispatched.
                tracing::error!("Audio system never became available; player will not be initialised");
                return;
            }

            let player = player::Player::new(conn_for_player, 0.7).await;
            {
                let mut p = player.lock().await;
                let s1 = state_clone.clone();
                let s2 = state_clone.clone();
                let s3 = state_clone.clone();
                let s4 = state_clone.clone();
                p.connect_progress_changed(move |progress| {
                    s1.dispatch(model::actions::Action::SetProgress(progress));
                });
                p.connect_track_changed(move |entry| {
                    s2.dispatch(model::actions::Action::SetPlayingTrack(entry));
                });
                p.connect_track_ended(move |entry| {
                    s3.dispatch(model::actions::Action::SetPlayedAt(entry.id, true));
                });
                p.connect_error(move |msg| {
                    s4.dispatch(model::actions::Action::AppendLog(
                        model::actions::LogEntry {
                            level: model::actions::LogLevel::Error,
                            message: msg,
                            timestamp: chrono::Utc::now().naive_utc(),
                        }
                    ));
                });
            }
            state_clone.set_player(player);
            tracing::info!("Player initialised successfully");
            state_clone.dispatch(model::actions::Action::InitVolume);
        });
    }

    ui.global::<Helper>().on_get_image(|image: ModelRc<i32>| {
        let decode = move || {
            let data_u8: Vec<u8> = image.iter().map(|v| v as u8).collect();
            let img = ImageReader::new(std::io::Cursor::new(data_u8))
                .with_guessed_format()
                .map_err(|e| slint::platform::PlatformError::from(format!("{:?}", e)))?
                .decode()
                .map_err(|e| slint::platform::PlatformError::from(format!("{:?}", e)))?;
            let (width, height) = img.dimensions();
            Ok(Image::from_rgb8(SharedPixelBuffer::clone_from_slice(
                img.as_bytes(),
                width,
                height,
            )))
        };
        decode().unwrap_or_else(|_: slint::platform::PlatformError| Image::default())
    });

    ui.run()?;

    Ok(())
}

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use std::sync::mpsc::sync_channel;

use image::{GenericImageView, ImageReader};
use slint::{Image, Model, ModelRc, SharedPixelBuffer};
use tracing::error;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

slint::include_modules!();

mod log_layer;
mod model;
mod view_model;

fn main() -> Result<(), Box<dyn Error>> {
    // Channel bridges the tracing layer (built before State) to State::dispatch (available after).
    let (log_tx, log_rx) = sync_channel::<model::actions::Action>(256);

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .with(log_layer::StateLogLayer::new(log_tx))
        .init();

    // Tokio runtime on background threads; Slint event loop stays on the main thread.
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    let (state, player_rx) = rt.block_on(async {
        let conn = database::connect().await.expect("DB connect failed");

        // Spawn Player::new on a background task so ALSA init doesn't block the UI from starting.
        let conn_for_player = conn.clone();
        let player_rx = tokio::spawn(async move {
            player::Player::new(conn_for_player, 0.7).await
        });

        let state = model::State::new(conn);
        (state, player_rx)
    });

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

    // Wire player callbacks once Player::new() finishes, then load library.
    {
        let state_clone = state.clone();
        tokio::spawn(async move {
            match player_rx.await {
                Ok(player) => {
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
                    state_clone.dispatch(model::actions::Action::InitVolume);
                    state_clone.dispatch(model::actions::Action::LoadLibraryEntry(0));
                }
                Err(e) => {
                    error!("Player init failed: {:?}", e);
                    std::process::exit(1);
                }
            }
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

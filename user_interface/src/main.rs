use std::sync::{Arc};
use std::sync::Mutex;
use gtk4::{Application, CssProvider, glib, IconTheme};
use gtk4::gdk::Display;
use gtk4::gio::resources_register_include;
use gtk4::glib::MainContext;
use gtk4::prelude::{ApplicationExt, ApplicationExtManual};
use tracing::{info};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::state::{Action, Dispatcher, Event, EventHandler, State};
use database::{connect, model::library_entry::Model as LibraryEntry};
use crate::components::{Component, WindowComponent};
use player::{Player, Progress};

mod state;
mod components;

const APP_ID: &str = "org.mupibox-rs.gui";

#[tokio::main]
async fn main() -> glib::ExitCode {
    tracing_subscriber::registry()
        .with(console_subscriber::spawn())
        .with(
            tracing_subscriber::fmt::layer().with_filter(
                Targets::new().with_default(LevelFilter::DEBUG).with_target("ureq", LevelFilter::INFO)
            )
        )
        .init();
    info!("Starting user interface");
    info!("Main running in {:?}", std::thread::current().id());

    resources_register_include!("composite_templates.gresource").expect("Failed to register resources.");


    let connection = connect().await.expect("Could not connect to database");
    let player = Player::new(&connection).await;
    let state = Arc::new(Mutex::new(State::new(connection).await));
    let dispatcher = Arc::new(Mutex::new(Dispatcher::new()));

    {
        let dispatcher1 = dispatcher.clone();
        let dispatcher2 = dispatcher.clone();
        let player = player.clone();
        glib::MainContext::default().spawn_local(async move {

            info!("MainContext spawn_local running in {:?}", std::thread::current().id());
            let mut player = player.lock().await;
            let handle_progress_change = move |progress: Progress| {
                dispatcher1
                    .lock()
                    .expect("Could not lock dispatcher")
                    .dispatch_action(Action::SetProgress(progress.as_f64()));
            };
            let handle_track_change = move |library_entry: Option<LibraryEntry>| {
                dispatcher2
                    .lock()
                    .expect("Could not lock dispatcher")
                    .dispatch_action(Action::SetPlayingTrack(library_entry.map(|entry| entry.id)));
            };

            player.connect_progress_change(handle_progress_change);
            player.connect_track_change(handle_track_change);
        });
    }

    let app = Application::builder().application_id(APP_ID).build();
    app.connect_startup(|_| {
        let theme = IconTheme::for_display(&Display::default().unwrap());
        theme.add_resource_path("/org/mupibox-rs/gui/icons/scalable/actions/");
        theme.add_search_path("/org/mupibox-rs/gui/icons/scalable/actions/");

        let provider = CssProvider::new();
        provider.load_from_resource("/org/mupibox-rs/gui/styles.css");

        gtk4::style_context_add_provider_for_display(
            &Display::default().expect("Could not connect to a display."),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    });
    app.connect_activate(move |app: &Application| {
        info!("Render app");
        let window = WindowComponent::new(state.clone(), dispatcher.clone(), None);
        window.present(app);

        {
            let window = Arc::new(Mutex::new(Box::new(window) as Box<dyn EventHandler>));
            let dispatcher_clone = dispatcher.clone();
            let state_clone = state.clone();
            let player = player.clone();

            dispatcher
                .lock()
                .expect("Could not lock dispatcher")
                .handle(
                    move |action| Action::process(action, state_clone.clone(), dispatcher_clone.clone(), player.clone()),
                    move |event| Event::broadcast(event, window.clone())
                );
        }

        // TODO connect to player progress and track change


        info!("Rendered");
    });

    // Run the application
    app.run()
}

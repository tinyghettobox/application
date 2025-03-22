use super::action::Action;
use crate::state::Event;
use gtk4::glib;
use std::future::Future;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tracing::{debug, error, warn};

pub struct Dispatcher {
    action_sender: UnboundedSender<Action>,
    event_sender: UnboundedSender<Event>,
    action_receiver: Option<UnboundedReceiver<Action>>,
    event_receiver: Option<UnboundedReceiver<Event>>,
}

impl Dispatcher {
    pub fn new() -> Self {
        let (action_sender, action_receiver) = unbounded_channel::<Action>();
        let (event_sender, event_receiver) = unbounded_channel::<Event>();
        Self {
            action_sender,
            event_sender,
            action_receiver: Some(action_receiver),
            event_receiver: Some(event_receiver),
        }
    }

    pub fn handle<A, AR, E>(&mut self, handle_action: A, handle_event: E)
    where
        A: (Fn(Action) -> AR) + 'static + Send,
        AR: Future<Output = ()> + Send,
        E: Fn(Event) + 'static,
    {
        let mut action_receiver = self
            .action_receiver
            .take()
            .expect("No action receiver yet set");
        let mut event_receiver = self
            .event_receiver
            .take()
            .expect("No event receiver yet set");

        tokio::spawn(async move {
            loop {
                match action_receiver.recv().await {
                    Some(action) => {
                        handle_action(action).await;
                    }
                    None => warn!("Failed to receive action from channel"),
                }
            }
        });

        glib::MainContext::default().spawn_local(async move {
            glib::idle_add_local(move || match event_receiver.try_recv() {
                Ok(event) => {
                    let event_name = format!("{:?}", event);
                    debug!("Received event {}", event_name);
                    handle_event(event);
                    glib::ControlFlow::Continue
                }
                Err(TryRecvError::Empty) => glib::ControlFlow::Continue,
                Err(TryRecvError::Disconnected) => glib::ControlFlow::Break,
            })
        });
    }

    pub fn dispatch_action(&self, action: Action) {
        debug!("Dispatching action {:?}", action);
        let action_sender = self.action_sender.clone();
        if let Err(error) = action_sender.send(action) {
            error!("Could not send action: {}", error);
        }
    }

    pub fn dispatch_event(&self, event: Event) {
        let event_name = format!("{:?}", event);
        debug!("Dispatching event {}", event_name);
        let event_sender = self.event_sender.clone();
        if let Err(error) = event_sender.send(event) {
            error!("Could not send event: {}", error);
        }
    }
}

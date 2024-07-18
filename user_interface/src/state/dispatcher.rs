use gtk4::glib;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender, UnboundedReceiver};
use tracing::{warn, error, info, debug};
use crate::state::Event;
use super::action::{Action};

pub struct Dispatcher {
    action_sender: UnboundedSender<Action>,
    event_sender: UnboundedSender<Event>,
    action_receiver: Option<UnboundedReceiver<Action>>,
    event_receiver: Option<UnboundedReceiver<Event>>
}

impl Dispatcher {
    pub fn new() -> Self {
        let (action_sender, action_receiver) = unbounded_channel::<Action>();
        let (event_sender, event_receiver) = unbounded_channel::<Event>();
        Self { action_sender, event_sender, action_receiver: Some(action_receiver), event_receiver: Some(event_receiver) }
    }

    pub fn handle(&mut self, handle_action: impl Fn(Action) + 'static, handle_event: impl Fn(Event) + 'static) {
        let mut action_receiver = self.action_receiver.take().expect("No action receiver yet set");
        let mut event_receiver = self.event_receiver.take().expect("No event receiver yet set");

        glib::MainContext::default().spawn_local(async move {
            loop {
                debug!("waiting for next action...");
                match action_receiver.recv().await {
                    Some(action) => {
                        debug!("handling action...");
                        handle_action(action);
                        debug!("finished action handling...");
                    },
                    None => warn!("Failed to receive action from channel")
                }
                debug!("finished action handling...");
            }
        });

        glib::MainContext::default().spawn_local(async move {
            loop {
                debug!("waiting for next event...");
                match event_receiver.recv().await {
                    Some(event) => {
                        debug!("event handling...");
                        handle_event(event);
                        debug!("finished event handling...");
                    },
                    None => warn!("Failed to receive action from channel")
                }
                debug!("event loop end...");
            }
        });
    }

    pub fn dispatch_action(&self, action: Action) {
        info!("Dispatching action {:?}", action);
        let action_sender = self.action_sender.clone();
        if let Err(error) = action_sender.send(action) {
            error!("Could not send action: {}", error);
        }
        info!("dispatched action");
    }

    pub fn dispatch_event(&self, event: Event) {
        info!("Dispatching event {:?}", event);
        let event_sender = self.event_sender.clone();
        if let Err(error) = event_sender.send(event) {
            error!("Could not send event: {}", error);
        }
        info!("dispatched event");
    }
}

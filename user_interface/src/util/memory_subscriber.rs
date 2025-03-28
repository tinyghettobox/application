use chrono::Local;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use tracing::field::{Field, Visit};
use tracing::{Event, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;

#[derive(Debug, Clone)]
pub struct LogMessage {
    pub time: String,
    pub target: String,
    pub message: String,
    pub level: String,
}

pub struct MemorySubscriber {
    messages: Arc<Mutex<Vec<LogMessage>>>,
}

impl MemorySubscriber {
    pub fn new(messages: Arc<Mutex<Vec<LogMessage>>>) -> Self {
        Self { messages }
    }
}

impl<S> Layer<S> for MemorySubscriber
where
    S: Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut messages = self.messages.lock().expect("could not lock");
        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);

        if !visitor.message.is_empty() {
            let msg = LogMessage {
                time: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                target: event.metadata().module_path().unwrap_or("").to_string(),
                level: event.metadata().level().to_string(),
                message: visitor.message.clone(),
            };
            // Keep size of messages small
            if messages.len() >= 1000 {
                messages.remove(0);
            }
            messages.push(msg);
        }
    }
}

#[derive(Default)]
struct MessageVisitor {
    message: String,
}

impl Visit for MessageVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
        }
    }
}

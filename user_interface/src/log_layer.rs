use std::sync::mpsc::SyncSender;

use tracing::Level;
use tracing_subscriber::Layer;

use crate::model::actions::{Action, LogEntry, LogLevel};

pub struct StateLogLayer {
    tx: SyncSender<Action>,
}

impl StateLogLayer {
    pub fn new(tx: SyncSender<Action>) -> Self {
        Self { tx }
    }
}

impl<S> Layer<S> for StateLogLayer
where
    S: tracing::Subscriber,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let level = match *event.metadata().level() {
            Level::ERROR => LogLevel::Error,
            Level::WARN => LogLevel::Warn,
            Level::INFO => LogLevel::Info,
            _ => LogLevel::Debug,
        };

        let mut visitor = MessageVisitor(String::new());
        event.record(&mut visitor);
        let message = if visitor.0.is_empty() {
            format!("[{}] {}", event.metadata().target(), event.metadata().name())
        } else {
            format!("[{}] {}", event.metadata().target(), visitor.0)
        };

        let entry = LogEntry {
            level,
            message,
            timestamp: chrono::Local::now().naive_local(),
        };
        self.tx.try_send(Action::AppendLog(entry)).ok();
    }
}

struct MessageVisitor(String);

impl tracing::field::Visit for MessageVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.0 = format!("{:?}", value);
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.0 = value.to_string();
        }
    }
}

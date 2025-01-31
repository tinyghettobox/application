use std::collections::VecDeque;
use std::ops::Add;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{sleep, spawn};
use std::time::{Duration, Instant};

use tracing::error;

struct Data<D> {
    pub triggered_at: Instant,
    pub data: D,
}

pub struct Debouncer<D> {
    buffer: Arc<Mutex<VecDeque<Data<D>>>>,
    sender: Sender<()>,
}

impl<D> Debouncer<D>
where
    D: Send + 'static,
{
    pub fn new<C: Fn(D) + Send + 'static>(delay: Duration, callback: C) -> Self {
        let (sender, receiver) = channel::<()>();
        let buffer = Arc::new(Mutex::new(VecDeque::<Data<D>>::new()));
        let buffer_ = buffer.clone();

        spawn(move || {
            for _ in receiver {
                let data = {
                    let mut buffer = buffer_.lock().unwrap();
                    let first = buffer.pop_front().expect("Debouncer triggered without buffered data");

                    if buffer.len() > 0 {
                        continue;
                    }
                    first
                };
                let wait_until = data.triggered_at.add(delay).duration_since(Instant::now());
                if wait_until > Duration::new(0, 0) {
                    sleep(wait_until);
                }
                if buffer_.lock().unwrap().len() > 0 {
                    continue;
                }

                callback(data.data);
            }
        });

        Debouncer { buffer, sender }
    }

    pub fn add(&self, data: D) {
        self.buffer.lock().unwrap().push_back(Data {
            triggered_at: Instant::now(),
            data,
        });
        if let Err(error) = self.sender.send(()) {
            error!("Could not notify debouncer thread: {}", error);
        }
    }
}

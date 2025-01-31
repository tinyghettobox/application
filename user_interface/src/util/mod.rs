pub mod debouncer;
pub mod ripple;

// pub struct DeadlockCheck {
//     stopped: Arc<AtomicBool>,
// }
//
// impl DeadlockCheck {
//     pub fn new(name: String) -> DeadlockCheck {
//         let stopped = Arc::new(AtomicBool::new(false));
//         let stopped_clone = stopped.clone();
//         std::thread::spawn(move || {
//             std::thread::sleep(std::time::Duration::from_secs(5));
//             if !stopped_clone.load(Ordering::Relaxed) {
//                 error!("Deadlock detected: {}", name)
//             } else {
//                 debug!("No deadlock {:?}", name)
//             }
//         });
//         DeadlockCheck { stopped }
//     }
// }

// impl Drop for DeadlockCheck {
//     fn drop(&mut self) {
//         debug!("DeadlockCheck dropped");
//         self.stopped.store(true, Ordering::Relaxed);
//     }
// }

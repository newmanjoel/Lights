use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::Notify;

#[derive(Debug, Clone)]
pub struct NotifyChecker {
    pub flag: Arc<AtomicBool>,
    pub notify: Arc<Notify>,
}

#[allow(dead_code)]
impl NotifyChecker {
    pub fn new() -> Self {
        Self {
            flag: Arc::new(AtomicBool::new(false)),
            notify: Arc::new(Notify::new()),
        }
    }

    pub fn _new_with_existing_notify(existing_notify: Arc<Notify>) -> Self {
        Self {
            flag: Arc::new(AtomicBool::new(false)),
            notify: existing_notify,
        }
    }

    pub fn set_notified(&self) {
        self.flag.store(true, Ordering::SeqCst);
        self.notify.notify_one();
    }

    pub fn is_notified(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }
}

pub async fn wait_for_signals(notify: NotifyChecker) {
    let mut interrupt = signal(SignalKind::interrupt()).unwrap();
    let mut terminate = signal(SignalKind::terminate()).unwrap();

    tokio::select! {
        _ = interrupt.recv() => println!("Received SIGINT, shutting down..."),
        _ = terminate.recv() => println!("Received SIGTERM, shutting down..."),
    }
    notify.set_notified();
    println!("Wait for Signal: Send and Stopping")
}

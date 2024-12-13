use chrono::{Local, Timelike};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::Notify;

use crate::command::ChangeLighting;
// use chrono::DateTime;

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

pub async fn timed_brightness(sender: tokio::sync::mpsc::Sender<ChangeLighting>, shutdown: NotifyChecker) {
    let night_brightness: u8 = 100;
    let day_brightness: u8 = 1;
    println!("Timed Brightness: Starting");
    while !shutdown.is_notified() {
        let now = Local::now();
        if now.time().hour() > 15 {
            sender.send(ChangeLighting::Brightness(night_brightness)).await.unwrap();
            // sender.send(night_brightness).await.unwrap();
        } else if now.time().hour() > 6 {
            sender.send(ChangeLighting::Brightness(day_brightness)).await.unwrap();
            // sender.send(day_brightness).await.unwrap();
        }
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
    println!("Timed Brightness: Stopped");
}

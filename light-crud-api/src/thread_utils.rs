use tokio::signal::unix::{signal, SignalKind};

#[derive(Debug)]
pub struct CompactSender<T> {
    pub sending_channel: tokio::sync::mpsc::Sender<T>,
    pub receving_channel: tokio::sync::mpsc::Receiver<T>,
}
impl<T> CompactSender<T> {
    pub fn new() -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel::<T>(32);
        CompactSender {
            sending_channel: tx,
            receving_channel: rx,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Notifier<T> {
    pub sending_channel: tokio::sync::watch::Sender<T>,
    pub receving_channel: tokio::sync::watch::Receiver<T>,
}
impl<T> Notifier<T> {
    pub fn new(initial_value: T) -> Self {
        let (tx, rx) = tokio::sync::watch::channel(initial_value);
        Notifier {
            sending_channel: tx,
            receving_channel: rx,
        }
    }
}

#[allow(dead_code)]
impl Notifier<bool> {
    pub fn new_flag() -> Self {
        return Self::new(false);
    }

    pub fn set_notified(&self) {
        self.sending_channel.send(true).unwrap();
    }

    pub async fn graceful_signal(mut self, wait_for_value: bool) -> () {
        self.receving_channel
            .wait_for(|value| *value == wait_for_value)
            .await
            .unwrap();
        return;
    }
}

pub async fn wait_for_signals(notify: Notifier<bool>) {
    let mut interrupt = signal(SignalKind::interrupt()).unwrap();
    let mut terminate = signal(SignalKind::terminate()).unwrap();

    tokio::select! {
        _ = interrupt.recv() => println!("Received SIGINT, shutting down..."),
        _ = terminate.recv() => println!("Received SIGTERM, shutting down..."),
    }
    notify.set_notified();
    println!("Wait for Signal: Send and Stopping")
}

use notify::{Config, Error, Event, INotifyWatcher, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};

pub type WatcherRx = Receiver<Result<Event, Error>>;
pub type SharedRx = Arc<Mutex<WatcherRx>>;

pub struct WatchHandler {}

impl WatchHandler {
    /// Setup the file watcher, watcher event loop and return the receiver channel for pushing notifications
    pub fn new(watch_dir: &str) -> (INotifyWatcher, SharedRx) {
        let (tx, mut rx) = mpsc::channel();

        let mut watcher = RecommendedWatcher::new(
            move |res| tx.send(res).expect("Failed to send event"),
            Config::default(),
        )
        .expect("Failed to create watcher");

        watcher
            .watch(Path::new(watch_dir), RecursiveMode::Recursive)
            .expect("Failed to watch dir");

        let shared_rx = Arc::new(Mutex::new(rx));

        (watcher, shared_rx)
    }
}

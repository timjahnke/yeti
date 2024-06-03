use notify::event::{AccessKind, AccessMode, ModifyKind};
use notify::{
    Config, Error, Event, EventKind, INotifyWatcher, RecommendedWatcher, RecursiveMode, Watcher,
};
use std::path::Path;
use std::process;
use std::sync::Arc;
use tokio::sync::mpsc::{channel, Receiver};
use tokio::sync::Mutex;

pub type WatcherRx = Receiver<Event>;
pub type SharedRx = Arc<Mutex<WatcherRx>>;

pub struct WatchHandler {}

impl WatchHandler {
    /// Setup the file watcher, watcher event loop and return the receiver channel for pushing notifications
    pub fn watcher(watch_dir: &str) -> (INotifyWatcher, SharedRx) {
        let (transmitter, receiver) = channel(1);

        let mut is_change_event_occuring = false;

        // Filter file events/ errors before sending into channel
        // Due to editor behaviour, multiple events are fired on a single file change
        let mut watcher = RecommendedWatcher::new(
            move |event: Result<Event, Error>| {
                match event {
                    Err(e) => {
                        eprintln!("Error: {:?}", e);
                        return;
                    }
                    Ok(event) => match event.kind {
                        // Send message on first modify event
                        EventKind::Modify(modify_kind) => match modify_kind {
                            ModifyKind::Data(_) => {
                                if !is_change_event_occuring {
                                    transmitter
                                        .blocking_send(event)
                                        .expect("Failed to send modify event");
                                    is_change_event_occuring = true;
                                }
                            }
                            // Ignore other Modification events. E.g.  create, metadata, rename, delete,
                            _ => {}
                        },
                        // Detect file access close event after save and clean up
                        EventKind::Access(access_kind) => match access_kind {
                            AccessKind::Close(AccessMode::Write) => {
                                is_change_event_occuring = false;
                            }
                            _ => {}
                        },
                        _ => {}
                    },
                };
            },
            Config::default(),
        )
        .expect("Failed to create watcher");

        match watcher.watch(Path::new(watch_dir), RecursiveMode::Recursive) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Failed to watch directory: {:?}", e);
                process::exit(1);
            }
        }

        let shared_rx = Arc::new(Mutex::new(receiver));

        (watcher, shared_rx)
    }
}

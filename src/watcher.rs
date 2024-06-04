use notify::event::{AccessKind, AccessMode, ModifyKind};
use notify::{
    Config, Error, Event, EventKind, INotifyWatcher, RecommendedWatcher, RecursiveMode, Watcher,
};
use std::path::Path;
use std::process;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

pub type WatcherRx = UnboundedReceiver<Event>;

pub struct WatchHandler {}

impl WatchHandler {
    /// Sets up the file watcher and an unbounded channel for publishing events from sync to async
    /// Returns a tuple of the watcher and `unbounded_channel` receiver
    pub fn watcher(watch_dir: &str) -> (INotifyWatcher, WatcherRx) {
        let (transmitter, receiver) = unbounded_channel();

        let mut is_change_event_occuring = false;

        // Filter file events/ errors before sending into channel
        // Due to editor behaviour, multiple events are fired on a single file change
        let mut watcher = RecommendedWatcher::new(
            move |event: Result<Event, Error>| {
                match event {
                    Err(e) => {
                        eprintln!("Error: {:?}", e);
                    }
                    Ok(event) => match event.kind {
                        // Send message on first modify event
                        EventKind::Modify(modify_kind) => match modify_kind {
                            ModifyKind::Data(_) => {
                                if !is_change_event_occuring {
                                    let is_scss_file = event
                                        .paths
                                        .iter()
                                        .any(|path| path.extension().unwrap_or_default() == "scss");

                                    // Only push .scss file changes
                                    if is_scss_file {
                                        transmitter
                                            .send(event)
                                            .expect("Failed to send modify event");
                                        is_change_event_occuring = true;
                                    }
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
                eprintln!("🚨 Failed to watch directory: {:?}", e);
                process::exit(1);
            }
        }

        (watcher, receiver)
    }
}

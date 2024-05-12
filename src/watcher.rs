use crate::server::SocketConnections;
use ::futures::{
    channel::mpsc::{channel, Receiver},
    executor, SinkExt, StreamExt,
};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    path::Path,
    sync::{Arc, Mutex},
};

pub type SharedWatcher = Arc<Mutex<RecommendedWatcher>>;

pub struct WatchHandler {
    pub watcher: SharedWatcher,
}

impl WatchHandler {
    pub async fn new() -> WatchHandler {
        let (watcher, _rx) = Self::async_watcher().expect("Failed to create file watcher");

        WatchHandler {
            watcher: Arc::new(Mutex::new(watcher)),
        }
    }

    pub fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)>
    {
        // Setup channel to watch for file events
        let (mut tx, rx) = channel(1);

        let watcher = RecommendedWatcher::new(
            move |res| {
                executor::block_on(async {
                    tx.send(res).await.unwrap();
                })
            },
            Config::default(),
        )?;

        Ok((watcher, rx))
    }

    pub async fn watch_files(
        &self,
        path: &Path,
        connections: SocketConnections,
    ) -> Result<(), notify::Error> {
        let (mut watcher, mut rx) = Self::async_watcher().expect("Failed to create file watcher");

        // Watch all files and directories recursively at path
        watcher
            .watch(path.as_ref(), RecursiveMode::Recursive)
            .expect("Failed to watch input path");
        println!("Watching: {:?}", path);

        while let Some(res) = rx.next().await {
            match res {
                Ok(event) => {
                    if event.kind.is_modify() {
                        println!(
                            "changed: {:?}",
                            event
                                .paths
                                .iter()
                                .map(|p| p.to_str().unwrap())
                                .collect::<Vec<_>>()
                                .join(", ")
                        );
                        // let mut tasks: Vec<> = Vec::new();

                        // for (addr, stream) in connections.lock().unwrap().iter() {
                        //     let task = tokio::spawn(async move {
                        //         let (mut sender, _receiver) = stream.split();

                        //         sender
                        //             .send(Message::Text("reload".to_string()))
                        //             .await
                        //             .expect(
                        //                 format!("Error sending notification to {:?}", addr).as_str(),
                        //             );
                        //     });

                        //     let (mut sender, _receiver) = stream.split();

                        //     sender
                        //         .send(Message::Text("reload".to_string()))
                        //         .await
                        //         .expect(format!("Error sending notification to {:?}", addr).as_str());
                        // }

                        println!("File modified. Reload notification sent. \n");
                    }
                }
                Err(e) => println!("watch error: {:?}", e),
            }
        }

        Ok(())
    }
}

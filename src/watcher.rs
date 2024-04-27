use ::futures::{
    channel::mpsc::{channel, Receiver},
    executor, SinkExt, StreamExt,
};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

use owo_colors::OwoColorize;

pub fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
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

pub async fn async_watch<P: AsRef<Path>>(
    path: P,
    ws_stream: WebSocketStream<TcpStream>,
) -> notify::Result<()> {
    let (mut watcher, mut rx) = async_watcher()?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

    let reload_message: &str = "reload";
    let notification_message = Message::Text(reload_message.to_string());

    let (mut sender, _receiver) = ws_stream.split();

    while let Some(res) = rx.next().await {
        match res {
            Ok(event) => {
                if event.kind.is_modify() {
                    println!("changed: {:?}", event);
                    sender.send(notification_message.clone()).await.unwrap();
                    println!("{}", "File modified. Reload notification sent. \n".green());
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}

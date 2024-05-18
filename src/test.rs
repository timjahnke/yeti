use axum::extract::ws::{Message as WsMessage, WebSocket};
use axum::prelude::*;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::thread;
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::{sync::broadcast, sync::mpsc};
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::WebSocketStream;

#[tokio::main]
async fn main() {
    // Create a channel for communication between the notify task and the axum server
    let (tx, _) = broadcast::channel(100);

    // Spawn the notify task
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        // Create a watcher object, delivering debounced events.
        let (tx, rx) = mpsc::channel();
        let mut watcher: RecommendedWatcher = Watcher::new_immediate(move |res| {
            tx.send(res).unwrap();
        })
        .unwrap();

        // Watch the current directory recursively for changes.
        watcher.watch(".", RecursiveMode::Recursive).unwrap();

        loop {
            match rx.recv().await {
                Ok(event) => {
                    // Send the event to all WebSocket clients
                    tx_clone.send(event).unwrap();
                }
                Err(e) => {
                    eprintln!("watch error: {:?}", e);
                    break;
                }
            }
        }
    });

    // Create an Axum server
    let app = route("/", get(|| async { "Hello, world!" })).route("/ws", get(websocket_handler));

    // Wrap the app in a middleware that checks for file change events
    let app = {
        let tx = Arc::new(Mutex::new(tx));
        axum::routing::nest(
            "/",
            axum::service::get(move || {
                let tx = tx.clone();
                async move {
                    // Create a receiver for file change events
                    let mut rx = tx.lock().unwrap().subscribe();

                    while let Ok(event) = rx.recv().await {
                        // Broadcast the file change event to all WebSocket clients
                        let message = serde_json::to_string(&event).unwrap();
                        broadcast_to_clients(message).await;
                    }
                }
            }),
        )
    };

    // Run the Axum server
    axum::Server::bind(&SocketAddr::from(([127, 0, 0, 1], 3000)))
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn websocket_handler(ws: WebSocket) {
    let (mut tx, _) = ws.split();

    // Keep WebSocket connection alive
    while let Some(Ok(msg)) = tx.recv().await {
        if msg.is_close() {
            break;
        }
    }
}

async fn broadcast_to_clients(message: String) {
    // Broadcast message to all WebSocket clients
    // Here, you would need to maintain a list of connected WebSocket clients
    // and send the message to each client.
}

// access existing file watcher and message on file event
//  executor::block_on(async {
//     while let Some(res) = rx
//         .lock()
//         .expect("Failed to acquire receiver during lock.")
//         .next()
//         .await
//     {
//         match res {
//             Err(e) => println!("Watch error: {:?}", e),
//             Ok(event) => {
//                 println!("Event: {:?}", event);
//                 if event.kind.is_modify() {
//                     println!(
//                         "File modified: {:?}",
//                         event
//                             .paths
//                             .iter()
//                             .map(|path| path.to_str().unwrap())
//                             .collect::<Vec<_>>()
//                             .join(", ")
//                     );

//                     // Iterate over connections and send reload notification
//                     // let mut connections_len = 0;
//                     // for (who, stream) in
//                     //     active_connections.connections.lock().unwrap().iter_mut()
//                     // {
//                     //     println!("Sending reload notification to: {:?}", who);
//                     //     stream
//                     //         .send(Message::Text("reload".to_string()))
//                     //         .await
//                     //         .expect(
//                     //             format!("Failed to send reload notification to {who}.")
//                     //                 .as_str(),
//                     //         );
//                     //     connections_len += 1;
//                     // }
//                     // println!("{connections_len} Reload notifications sent. \n");
//                 }
//             }
//         }
//     }
// });

use ::futures::executor;
use owo_colors::OwoColorize;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::net::TcpStream;
use tokio_tungstenite::accept_async;

use crate::watcher::async_watch;

pub type _PeerMap = Arc<Mutex<HashMap<u32, SocketAddr>>>;

/**
 * Handles incoming socket connections. Is passed to the stream listener.
 */
pub async fn handle_connection(stream: TcpStream, addr: SocketAddr) {
    println!("{}", format!("Incoming connection from: {}", addr).yellow());

    let ws_stream = accept_async(stream)
        .await
        .expect("Error during websocket handshake occurred");

    println!("{}", format!("Connection established: {} \n", addr).green());

    // Setup file watcher
    let input_path = std::env::args()
        .nth(1)
        .expect("Argument 1 needs to be a path");

    println!(
        "{}",
        format!("Watching: {input_path}").truecolor(251, 146, 60)
    );

    executor::block_on(async {
        if let Err(e) = async_watch(input_path, ws_stream).await {
            println!("watch error: {:?}", e);
        }
    });

    // println!("Sending notification in 3 seconds \n");
    // tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // sender.send(notification_message.clone()).await.unwrap();

    // if let Some(ws) = peer_map.lock().unwrap().get(&addr) {
    //     // if let Err(e) = ws.send(notification_message).await {
    //     match ws.send(notification_message.clone()) {
    //         Ok(_) => println!("Notification sent to {}. \n", addr),
    //         Err(e) => println!("Error sending notification to {}: {:?}", addr, e),
    //     }
    // }

    println!("{} disconnected \n", &addr);
    // peer_map.lock().unwrap().remove(&addr);
}

// pub fn find_available_port(base_addr: &str, mut port: u16) -> Option<u16> {
//     let mut counter = 0;
//     let max_attempts = 6;
//     loop {
//         let socket_addr: SocketAddr = format!("{base_addr}:{port}")
//             .parse()
//             .expect(format!("Failed to parse socket address {base_addr}:{port}").as_str());

//         match TcpListener::bind(socket_addr) {
//             Ok(_) => {
//                 println!("Binding to available port: {}", port);
//                 return Some(port);
//             }
//             Err(_) => {
//                 match counter {
//                     // If we've tried the max number of attempts, return None
//                     x if x >= max_attempts => {
//                         println!("Max attempts reached! Stopped port number search.");
//                         return None;
//                     }
//                     _ => {
//                         // Port is in use, try the next one and update counter
//                         port += 1;
//                         counter += 1;
//                     }
//                 }
//             }
//         }
//     }
// }

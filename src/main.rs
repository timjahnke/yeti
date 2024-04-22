use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

// use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};

use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

// Color terminal output
use owo_colors::OwoColorize;

#[tokio::main]
async fn main() {
    // Setup server to listen on localhost port 8000
    let addr: &str = "127.0.0.1";
    let port: &str = "8000";

    // Satisfy SocketAddr type. E.g. {}:{}""
    let port_addr: SocketAddr = format!("{addr}:{port}")
        .parse()
        .expect("Failed to parse socket address.");

    let state = PeerMap::new(Mutex::new(HashMap::new()));

    // Setup event loop and TCP listener for connections
    let listener = TcpListener::bind(&&port_addr)
        .await
        .expect(format!("Failed to bind listener to address {port_addr}").as_str());

    let input_dir = "src/scss";
    // let output_dir = "dist/css";

    println!("\n");
    println!("{}", "🦀 Rust WebSockets v1.0".truecolor(251, 146, 60));
    println!("🔌 WebSockets Server running... \n");
    println!("🚀 Server details:");
    println!(
        "   {} {}",
        "Address:".green(),
        &port_addr.green().underline()
    );

    println!(
        "{}",
        format!("   Socket Address: ws://localhost:{port}").truecolor(52, 211, 153)
    );

    // println!("⌚ Watching for SCSS file changes in: {}", input_dir);
    // TODO: Display in console active connections

    // Setup socket server to listen for incoming connections
    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(state.clone(), stream, port_addr));
    }
}

async fn handle_connection(peer_map: PeerMap, stream: TcpStream, addr: SocketAddr) {
    println!("Incoming connection from: {} \n", addr);

    let ws_stream = accept_async(stream)
        .await
        .expect("Error during websocket handshake occurred");

    println!("WebSocket connection established: {} \n", addr);

    // Insert the write part of this peer ot the peer map.
    let (tx, rx) = unbounded_channel();
    peer_map.lock().unwrap().insert(addr, tx);

    let (outgoing, incoming) = ws_stream.split();

    let notification_message = Message::Text("Hello from the server".to_string());

    println!("Sending notification in 5 seconds \n");
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    if let Some(ws) = peer_map.lock().unwrap().get(&addr) {
        // if let Err(e) = ws.send(notification_message).await {
        match ws.send(notification_message.clone()) {
            Ok(_) => println!("Notification sent to {}. \n", addr),
            Err(e) => println!("Error sending notification to {}: {:?}", addr, e),
        }
    }

    println!("{} disconnected \n", &addr);
    peer_map.lock().unwrap().remove(&addr);
}

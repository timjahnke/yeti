use std::{
    collections::HashMap,
    net::SocketAddr,
    path::Path,
    sync::{Arc, Mutex},
};

use futures::executor;
use tokio::net::TcpStream;
use tokio_tungstenite::{accept_async, WebSocketStream};

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::{extract::connect_info::ConnectInfo, response::Response};

use crate::watcher::SharedWatcher;

pub type SocketConnections = Arc<Mutex<HashMap<SocketAddr, WebSocket>>>;

#[derive(Clone)]
pub struct ServerHandler {
    pub connections: SocketConnections,
}
impl ServerHandler {
    pub async fn new() -> Self {
        let connections = Arc::new(Mutex::new(HashMap::new()));
        Self { connections }
    }

    pub async fn ws_handler(
        self,
        ws: WebSocketUpgrade,
        ConnectInfo(addr): ConnectInfo<SocketAddr>,
        watcher: SharedWatcher,
    ) -> Response {
        println!("ws handler fired");
        ws.on_upgrade(move |socket| Self::handle_socket(self, socket, addr))
    }

    pub async fn handle_socket(self, socket: WebSocket, who: SocketAddr) {
        println!("Incoming connection from {:?}", who);

        self.connections.lock().unwrap().insert(who, socket);

        // Store each connection in the hashmap

        // access existing file watcher and message on file event

        executor::block_on(async {})

        //  Setup and attach file watcher to stream
        // executor::block_on(async {
        //     match watch_files(
        //         Path::new(&watch_dir),
        //         active_connections.connections.clone(),
        //     )
        //     .await
        //     {
        //         Err(e) => println!("Watch error: {:?}", e),
        //         Ok(_) => {
        //             println!("File watcher running...");
        //             // Setup socket server to listen for incoming connections
        //             while let Ok((stream, _)) = listener.accept().await {
        //                 println!("the stream {:?}", stream);
        //                 tokio::spawn(handle_connection(
        //                     stream,
        //                     active_connections.connections.clone(),
        //                 ));
        //             }
        //         }
        //     }
        // });

        //

        // socket
        //     .send(Message::Text(("asdfadsf".to_string())))
        //     .await
        //     .unwrap();
    }
}

/**
 * Handles and processes incoming socket connections. Is passed to the stream listener.
 */
pub async fn handle_connection(stream: TcpStream, connections: SocketConnections) {
    println!("the stream: {:?}", stream);

    let client_addr = stream.peer_addr().expect("Failed to get peer address");
    println!("Incoming connection from: {client_addr}");

    let ws_stream = accept_async(stream)
        .await
        .expect("Error during websocket handshake occurred");

    // connections.lock().unwrap().insert(client_addr, ws_stream);
    println!("{}", format!("Connection established: {client_addr} \n"));

    println!("{} disconnected \n", &client_addr);
    // connections.lock().unwrap().remove(&client_addr);
}

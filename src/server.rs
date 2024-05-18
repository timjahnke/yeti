use std::net::SocketAddr;

use axum::extract::ws::{WebSocket, WebSocketUpgrade};
use axum::{extract::connect_info::ConnectInfo, response::Response};
use tokio_tungstenite::tungstenite::accept;

use crate::watcher::SharedRx;

#[derive(Clone)]
pub struct ServerHandler {}
impl ServerHandler {
    pub async fn new() -> Self {
        Self {}
    }

    pub async fn ws_handler(
        self,
        ws: WebSocketUpgrade,
        ConnectInfo(addr): ConnectInfo<SocketAddr>,
        rx: SharedRx,
    ) -> Response {
        println!("ws handler fired");
        ws.on_upgrade(move |socket| Self::handle_socket(self, socket, addr, rx))
    }

    ///Handles and processes incoming socket connections. Is passed to the stream listener.
    pub async fn handle_socket(self, mut socket: WebSocket, who: SocketAddr, rx: SharedRx) {
        println!("Incoming connection from {:?}", who);

        println!("Starting outer loop");

        // Create task to check socket connection
        let socket_task = tokio::spawn(async move {
            while let Some(msg) = socket.recv().await {
                let msg = match msg {
                    Ok(msg) => msg,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        break;
                    }
                };
            }
        });

        // Create task for watching file events
        let watch_task = tokio::spawn(async move {
            loop {
                let event = rx.lock().unwrap().recv().unwrap();
                match event {
                    Ok(event) => {
                        println!("Event: {:?}", event);
                    }
                    Err(e) => {
                        eprintln!("Error: {:?}", e);
                        break;
                    }
                }
            }
        });

        // If either task fails, kill both tasks
        tokio::select! {
            _ = socket_task => {
                println!("Socket task ended");
            },
            _ = watch_task => {
                println!("Watch task ended");
            }
        }

        println!("Connection closed: {}", who);
    }
}

use std::net::SocketAddr;
use std::time::Duration;

use axum::extract::ws::{WebSocket, WebSocketUpgrade};
use axum::{extract::connect_info::ConnectInfo, response::Response};
use tokio::time::sleep;

use crate::watcher::SharedRx;

#[derive(Clone)]
pub struct ServerHandler {}

impl ServerHandler {
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

        // Create task to check socket connection
        let socket_task = tokio::spawn(async move {
            while let Some(msg) = socket.recv().await {
                match msg {
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
                println!("Server watch task loop fired");

                // Blocks thread and waits for next message in channel
                let event = rx.lock().unwrap().recv();
                match event {
                    Ok(event) => {
                        if event.kind.is_modify() {
                            println!("Modify Event: {:?}", event.kind);

                            // Build the sass here
                            println!("File change. Building Sass... 10s delay.");
                            sleep(Duration::from_secs(5)).await;
                            println!("Sass built!")
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {:?}", e);
                    }
                }
            }

            // Or run the sass watch command
            // Kill command on socket close below
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

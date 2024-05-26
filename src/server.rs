use std::net::SocketAddr;
use std::process::Command;
use std::time::Duration;

use axum::extract::ws::{WebSocket, WebSocketUpgrade};
use axum::{extract::connect_info::ConnectInfo, response::Response};
use tokio::time::sleep;

use crate::config::ServerConfig;
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

        // Read from JSON
        let config_filename = "yeti.json";
        let ServerConfig {
            input_file_path,
            output_file_path,
            ..
        } = ServerConfig::read_json(&config_filename);

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

                            // E.g. sass assets/styles/main.scss:dist/main.css --style=compressed
                            let sass_command = Command::new("sass")
                                .args([
                                    format!("{input_file_path}:{output_file_path}").as_str(),
                                    "--style=compressed",
                                ])
                                .output();

                            match sass_command {
                                Ok(res) => {
                                    let formatted_output = String::from_utf8(res.stdout)
                                        .expect("Failed to parse Sass output");
                                    println!("Sass output: {}", formatted_output);
                                }
                                Err(e) => {
                                    eprintln!("Error building Sass: {:?}", e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {:?}", e);
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

use std::net::SocketAddr;
use std::process::Command;
use std::time::SystemTime;

use axum::extract::ws::{WebSocket, WebSocketUpgrade};
use axum::{extract::connect_info::ConnectInfo, response::Response};
use notify::event::ModifyKind;
use notify::EventKind;

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
        ws.on_upgrade(move |socket| Self::handle_socket(self, socket, addr, rx))
    }

    ///Handles and processes incoming socket connections. Is passed to the stream listener.
    pub async fn handle_socket(self, mut socket: WebSocket, who: SocketAddr, rx: SharedRx) {
        println!("Incoming connection from {:?} \n", who);

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
                // Blocks thread and waits for next message in channel
                let event = rx.lock().unwrap().recv();
                match event {
                    Ok(event) => match event.kind {
                        EventKind::Modify(modify_kind) => match modify_kind {
                            ModifyKind::Data(_) => {
                                println!("File change. Building Sass...");

                                // E.g. sass assets/styles/main.scss:dist/main.css --style=compressed
                                // Build args, pass to stdout and spawn process for execution
                                let now = SystemTime::now();
                                let sass_output = Command::new("sass")
                                    .args([
                                        format!("{input_file_path}:{output_file_path}").as_str(),
                                        "--style=compressed",
                                        "--no-source-map",
                                    ])
                                    .output();
                                let elapsed = now.elapsed().unwrap();

                                match sass_output {
                                    Ok(_res) => {
                                        println!(
                                            "Built Sass in {:?}ms",
                                            elapsed.as_millis() as f64 / 1000.0
                                        );
                                    }
                                    Err(e) => {
                                        eprintln!("Error building Sass: {:.3}", e);
                                    }
                                }
                            }
                            // Ignore any other modification kind. e.g. Date, metadata, name
                            _ => {}
                        },
                        _ => {}
                    },

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

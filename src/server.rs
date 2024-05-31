use std::io::{BufRead, BufReader};
use std::net::SocketAddr;
use std::process::{Command, Stdio};
use std::time::SystemTime;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::{extract::connect_info::ConnectInfo, response::Response};
use futures_util::{SinkExt, StreamExt};
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

    /// Handles and processes incoming socket connections. Is passed to the stream listener.
    pub async fn handle_socket(self, socket: WebSocket, who: SocketAddr, rx: SharedRx) {
        println!("Incoming connection from {:?}", who);

        let config_filename = "yeti.json";
        let ServerConfig {
            input_file_path,
            output_file_path,
            ..
        } = ServerConfig::read_json(&config_filename);

        // Split the socket into two: sender and receiver
        // Also fixes ownership problem when moved within task closures
        let (mut sender, mut receiver) = socket.split();

        // Create task to check socket connection
        // While able to receive a message from the client, connection is deemed active
        // * Client will never send a message, though the websocket server does not know and will wait *
        let socket_task = tokio::spawn(async move {
            while let Some(msg) = receiver.next().await {
                match msg {
                    Ok(_) => {}
                    Err(e) => eprintln!("{e}"),
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

                                let now = SystemTime::now();
                                let sass_cmd = Command::new("sass")
                                    .args([
                                        "--stop-on-error",
                                        format!("{input_file_path}:{output_file_path}").as_str(),
                                        "--style=compressed",
                                        "--no-source-map",
                                    ])
                                    .stdout(Stdio::piped())
                                    .spawn();

                                let elapsed = now.elapsed().unwrap();

                                match sass_cmd {
                                    Ok(mut child) => {
                                        let stdout = child.stdout.take().unwrap();
                                        let lines = BufReader::new(stdout).lines();
                                        let mut count = 0;

                                        // A successful sass build does not emit to stdout, errors do instead
                                        // If there's an error, iterate it and increment count
                                        lines
                                            .inspect(|_| count += 1)
                                            .for_each(|line| println!("{}", line.unwrap()));

                                        // If there are no errors, send a reload message to the client
                                        if count == 0 {
                                            println!(
                                                "Built Sass in {:.3}ms",
                                                elapsed.as_millis() as f64 / 1000.0
                                            );
                                            match sender
                                                .send(Message::Text("reload".to_string()))
                                                .await
                                            {
                                                Ok(_) => {}
                                                Err(e) => {
                                                    eprintln!(
                                                        "🚨 Failed to send reload message. {e}"
                                                    )
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Error building Sass: {e}");
                                    }
                                }
                            }
                            // Ignore any other modification kind. e.g. Date, metadata, name
                            _ => {}
                        },
                        // Ignore other file events. Is also filtered before by watcher
                        _ => {}
                    },

                    Err(e) => {
                        eprintln!("Error: {:?}", e);
                    }
                }
            }
        });

        // When the client disconnects, the socket task is treated as completed
        // The watcher task is then dropped as it never completes
        tokio::select! {
            _ = socket_task => {
                println!("Socket task resolved. Client disconnected.");
            },
            _ = watch_task => {
                println!("Watch task resolved");
            }
        }

        println!("Connection closed: {} \n", who);
    }
}

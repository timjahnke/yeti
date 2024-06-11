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

    /// Handles and processes incoming socket connections and assigns them a file watcher.
    pub async fn handle_socket(self, socket: WebSocket, who: SocketAddr, shared_rx: SharedRx) {
        println!("🤝 Incoming connection from {:?}", who);

        let config_filename = format!(
            "{}/yeti.json",
            std::env::current_dir()
                .expect("Couldn't get current directory")
                .display()
        );
        let ServerConfig {
            input_file_path,
            output_file_path,
            stop_on_error,
            ..
        } = ServerConfig::read_json(&config_filename);

        // Split socket to monitor send/ receive in parallel
        // Also for ownership and to move within task closures
        let (mut sender, mut receiver) = socket.split();

        // Create task to check socket connection
        // While able to wait to receive a message from the client, connection is deemed active
        let mut socket_task = tokio::spawn(async move {
            while let Some(msg) = receiver.next().await {
                match msg {
                    // Client will never send a message, though the server doesn't know and will wait
                    Ok(_) => {}
                    Err(e) => eprintln!("{e}"),
                };
            }
        });

        // Create task for watching file events
        let mut watch_task = tokio::spawn(async move {
            // Channel will sleep until message in channel
            while let Some(event) = shared_rx.lock().await.recv().await {
                match event.kind {
                    EventKind::Modify(modify_kind) => match modify_kind {
                        ModifyKind::Data(_) => {
                            println!("🔨 File change. Building Sass...");

                            let now = SystemTime::now();
                            let sass_cmd = Command::new("sass")
                                .args([
                                    if stop_on_error { "--stop-on-error" } else { "" },
                                    format!("{input_file_path}:{output_file_path}").as_str(),
                                    "--style=compressed",
                                    "--no-source-map",
                                ])
                                .stderr(Stdio::piped())
                                .spawn();

                            let elapsed = now.elapsed().unwrap();

                            match sass_cmd {
                                Ok(mut child) => {
                                    let stderr = child.stderr.take().unwrap();
                                    let lines = BufReader::new(stderr).lines();
                                    let mut count = 0;

                                    // If there's an error, iterate it and increment count
                                    lines
                                        .inspect(|_| count += 1)
                                        .for_each(|line| println!("{}", line.unwrap()));

                                    // If there are no errors, send a reload message to the client
                                    if count == 0 {
                                        println!(
                                            "✅ Built Sass in {:.3}ms",
                                            elapsed.as_millis() as f64 / 1000.0
                                        );

                                        match sender.send(Message::Text("reload".to_string())).await
                                        {
                                            Ok(_) => {
                                                println!("✅ Successfully sent reload message!");
                                            }
                                            Err(e) => {
                                                eprintln!("🚨 Failed to send reload message. {e}")
                                            }
                                        }
                                    } else {
                                        eprintln!("❌ Cancelled Sass build. ");
                                    }
                                }
                                Err(e) => {
                                    eprintln!("🚨 Error building Sass: {e}");
                                }
                            }
                        }
                        // Ignore any other modification kind. e.g. Date, metadata, name
                        _ => {}
                    },
                    // Ignore other file events. Is also filtered before by watcher
                    _ => {}
                }
            }
        });

        // When one task exits, abort the other
        tokio::select! {
            // When the client disconnects, the socket task exits
            _ = &mut socket_task => {
                watch_task.abort();
                println!("👋 Watch aborted. Connection closed: {} \n", who);
            },
            // Watcher should not resolve unless error
            _ = &mut watch_task => {
                println!("🚨 Watch task resolved");
                socket_task.abort();
            }
        }
    }
}

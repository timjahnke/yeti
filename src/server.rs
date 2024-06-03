use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::net::SocketAddr;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::SystemTime;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::{extract::connect_info::ConnectInfo, response::Response};
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use notify::event::ModifyKind;
use notify::EventKind;
use tokio::sync::Mutex;

use crate::config::ServerConfig;
use crate::watcher::SharedRx;

type SharedConnections = Arc<Mutex<HashMap<SocketAddr, SplitSink<WebSocket, Message>>>>;

#[derive(Clone)]
pub struct ServerHandler {
    pub connections: Arc<Mutex<HashMap<SocketAddr, SplitSink<WebSocket, Message>>>>,
}

impl ServerHandler {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn ws_handler(
        self,
        ws: WebSocketUpgrade,
        ConnectInfo(addr): ConnectInfo<SocketAddr>,
        rx: SharedRx,
        connections: SharedConnections,
    ) -> Response {
        ws.on_upgrade(move |socket| Self::handle_socket(self, socket, addr, rx, connections))
    }

    /// Handles and processes incoming socket connections. Is passed to the stream listener.
    pub async fn handle_socket(
        self,
        socket: WebSocket,
        who: SocketAddr,
        rx: SharedRx,
        connections: SharedConnections,
    ) {
        println!("Incoming connection from {:?}", who);

        let config_filename = format!(
            "{}/yeti.json",
            std::env::current_dir()
                .expect("Couldn't get current directory")
                .display()
        );
        let ServerConfig {
            input_file_path,
            output_file_path,
            ..
        } = ServerConfig::read_json(&config_filename);

        let (sender, mut receiver) = socket.split();
        connections.lock().await.insert(who, sender);

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

        let watcher_connections = connections.clone();
        // Create task for watching file events
        // await for its execution to prevent the watcher from dropping due to lifetime of its function block
        let watch_task = tokio::spawn(async move {
            while let Some(event) = rx.lock().await.recv().await {
                match event.kind {
                    EventKind::Modify(modify_kind) => match modify_kind {
                        ModifyKind::Data(data_event) => {
                            println!("Data event: {:?}", data_event);
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
                                            "Built Sass in {:.3}ms \n",
                                            elapsed.as_millis() as f64 / 1000.0
                                        );

                                        let mut res = watcher_connections.lock().await;
                                        let sender = res.get_mut(&who).expect(
                                            format!("🚨 No connection found for {who}").as_str(),
                                        );
                                        match sender.send(Message::Text("reload".to_string())).await
                                        {
                                            Ok(_) => {}
                                            Err(e) => {
                                                eprintln!("🚨 Failed to send reload message. {e}")
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
                }
            }
        });
        // .await
        // .unwrap();

        // When the client disconnects, the socket task is treated as completed
        // The watcher task is then dropped
        tokio::select! {
            _ = socket_task => {
                let mut connected_clients = connections.lock().await;
                let _ = connected_clients.remove(&who).expect(format!("No connection found for {}", who).as_str());
                println!("Socket task resolved. Client disconnected.");

            },
            _ = watch_task => {
                println!("Watch task resolved");
            }
        }

        println!("Watch cancelled. Connection closed: {} \n", who);
    }
}

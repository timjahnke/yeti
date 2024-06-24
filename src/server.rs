use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::net::SocketAddr;
use std::process::{Command, Stdio};

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::{extract::connect_info::ConnectInfo, response::Response};
use futures_util::{SinkExt, StreamExt};
use notify::event::ModifyKind;
use notify::EventKind;
use tokio::time::Instant;

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
        config: ServerConfig,
    ) -> Response {
        ws.on_upgrade(move |socket| Self::handle_socket(self, socket, addr, rx, config))
    }

    /// Handles and processes incoming socket connections and assigns them a file watcher.
    pub async fn handle_socket(
        self,
        socket: WebSocket,
        who: SocketAddr,
        shared_rx: SharedRx,
        config: ServerConfig,
    ) {
        println!("🤝 Incoming connection from {:?}", who);

        let ServerConfig {
            input_file_path,
            output_file_path,
            stop_on_error,
            experimental,
            ..
        } = config;

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
                // Only process data modification events
                if matches!(event.kind, EventKind::Modify(ModifyKind::Data(_))) {
                    println!("🔨 File change. Building Sass...");

                    let now = Instant::now();
                    // Use Grass crate for Sass compilation
                    if experimental {
                        println!("🌱 Using Grass to build Sass...");
                        let css_result = grass::from_path(
                            &input_file_path,
                            &grass::Options::default().style(grass::OutputStyle::Compressed),
                        );

                        if let Ok(css) = css_result {
                            File::create(&output_file_path)
                                .unwrap()
                                .write_all(css.as_bytes())
                                .expect("Failed to write CSS to output css file");

                            println!(
                                "✅ Built Sass in {:.3}ms",
                                now.elapsed().as_millis() as f64 / 1000.0
                            );

                            match sender.send(Message::Text("reload".to_string())).await {
                                Ok(_) => println!("✅ Successfully sent reload message! \n"),
                                Err(e) => eprintln!("🚨 Failed to send reload message. {e}"),
                            }
                        } else if let Err(e) = css_result {
                            eprintln!("🚨 Error building Sass: {e}");
                        }
                    }
                    // Default Dart Sass compilation via CLI
                    else {
                        let input_output_path = format!("{input_file_path}:{output_file_path}");

                        // Build Args based on yeti.json configuration
                        let mut dynamic_args: Vec<&str> = vec![
                            if stop_on_error { "--stop-on-error" } else { "" },
                            &input_output_path,
                            "--style=compressed",
                            "--no-source-map",
                        ];

                        // Discard empty args
                        dynamic_args.retain(|&arg| !arg.is_empty());

                        let sass_cmd = Command::new("sass")
                            .args(dynamic_args)
                            .stderr(Stdio::piped())
                            .spawn();

                        if let Ok(mut child) = sass_cmd {
                            let stderr = child.stderr.take().unwrap();
                            let lines = BufReader::new(stderr).lines();
                            let mut count = 0;

                            if stop_on_error {
                                // Don't increment count in stop_on_error config option
                                lines.for_each(|line| eprintln!("{}", line.unwrap()));
                            } else {
                                // If there's an error, iterate it and increment count
                                lines
                                    .inspect(|_| count += 1)
                                    .for_each(|line| eprintln!("{}", line.unwrap()));
                            }

                            // If there are no errors, send a reload message to the client
                            if count == 0 || !stop_on_error {
                                println!(
                                    "✅ Built Sass in {:.3}ms",
                                    now.elapsed().as_millis() as f64 / 1000.0
                                );

                                match sender.send(Message::Text("reload".to_string())).await {
                                    Ok(_) => println!("✅ Successfully sent reload message! \n"),
                                    Err(e) => eprintln!("🚨 Failed to send reload message. {e}"),
                                }
                            } else {
                                eprintln!("❌ Cancelled Sass build. ");
                            }
                        } else if let Err(e) = sass_cmd {
                            eprintln!("🚨 Error building Sass: {e}");
                        }
                    }
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

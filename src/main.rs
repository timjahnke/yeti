use axum::http::header;
use axum::routing::get;
use axum::Router;
use std::fs;
use std::net::{IpAddr, Ipv4Addr};
use std::process::{self, Command};
use std::{env, net::SocketAddr};
use tokio::net::TcpListener;
use watcher::WatchHandler;

mod config;
mod server;
mod watcher;

use crate::config::ServerConfig;
use crate::server::ServerHandler;

#[tokio::main]
async fn main() {
    println!("🧊 Yeti v{}", env!("CARGO_PKG_VERSION"));

    // Check sass is installed in global path on the current system
    let sass_output = Command::new("sass")
        .arg("--version")
        .output()
        .unwrap_or_else(|e| {
            eprintln!("   Sass executable not found. \nError: {:?}", e);
            process::exit(1);
        });

    let sass_version = String::from_utf8(sass_output.stdout).unwrap();
    println!("   Sass installed! Version {}", sass_version);

    let json_path = std::env::current_dir()
        .unwrap_or_else(|e| {
            eprintln!("🚨 Failed to get current directory. Error: {:?}", e);
            process::exit(1);
        })
        .join("yeti.json");

    // Safely exit if no yeti.json file is found
    if !json_path.exists() {
        eprintln!("🚨 No yeti.json file found in the current directory. Create one to begin. Exiting... \n");
        process::exit(1);
    }

    // Check for existing empty json file (0 bytes in size)
    let is_json_empty = fs::metadata(&json_path)
        .expect("Failed to read yeti.json metadata")
        .len()
        == 0;

    // Set default values for empty yeti.json and exit
    if is_json_empty {
        ServerConfig::set_default_json_values(&json_path);
        println!("📝 Set default yeti.json key-value pairs. Update the values and re-run yeti. Exiting... \n");
        process::exit(1);
    }

    // Read yeti.json for configuration values
    let json_config = ServerConfig::read_json(&json_path);
    let server_config = ServerConfig::new(json_config);

    if server_config.experimental {
        println!("🚧 Experimental mode enabled. Using Grass to compile Sass files.");
    }

    // Initialise Server Handler instance
    let server_handler = ServerHandler {};

    // Create socket port address
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), server_config.port);

    // Setup listener and app for web server router
    let listener = TcpListener::bind(socket).await.unwrap_or_else(|e| {
        eprintln!(
            "🚨 Port:{} is in use. Try another port. \nError: {e}",
            server_config.port
        );
        process::exit(1);
    });

    // Instantiate file watcher and share receiver to event channel
    let (_watcher, watcher_rx) = WatchHandler::watcher(&server_config.watch_dir);

    // Initialise shared file watcher & channel receiver
    println!("🔭 Watching directory /{}... \n", &server_config.watch_dir);
    println!("✨ WebSockets Server active... \n");
    println!("🏠 Host Address: ");
    println!("   IP: {}", socket.ip());
    println!("   Socket: ws://localhost:{}/ws \n", socket.port());

    let yeti_state = server_config.clone();

    // Pass receiver to Web Socket route handler and start the web server
    axum::serve(
        listener,
        Router::new()
            .route(
                "/ws",
                get(move |ws, connect_info| {
                    server_handler
                        .clone()
                        .ws_handler(ws, connect_info, watcher_rx, yeti_state)
                }),
            )
            .route(
                "/client",
                get(move || async move {
                    (
                        [(header::CONTENT_TYPE, "text/javascript")],
                        ServerConfig::serve_javascript_string(
                            socket.port(),
                            &server_config.style_tag_id,
                        ),
                    )
                }),
            )
            .into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("Could not server web server");
}

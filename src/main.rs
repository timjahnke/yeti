use axum::routing::get;
use axum::Router;
use std::fs;
use std::path::Path;
use std::process::{self, Command};
use std::{env, net::SocketAddr};
use tokio::net::TcpListener;
use tower_http::services::ServeFile;

mod config;
mod server;
mod watcher;

use crate::config::ServerConfig;
use crate::server::ServerHandler;
use crate::watcher::WatchHandler;

#[tokio::main]
async fn main() {
    let config_filename = "yeti.toml";
    println!("🧊 Yeti v{}", env!("CARGO_PKG_VERSION"));

    // Check sass is installed in global path on the current system
    match Command::new("sass").arg("--version").output() {
        Ok(res) => {
            let formatted_output =
                String::from_utf8(res.stdout).expect("Failed to parse Sass output");
            println!("   Sass installed! Version {}", formatted_output);
        }
        Err(e) => {
            eprintln!("   Sass executable not found. \nError: {:?}", e);
            process::exit(1);
        }
    }

    // Gracefully exit if no yeti.toml file is found
    if !Path::new(&config_filename).exists() {
        eprintln!("🚨 No yeti.toml file found in the current directory. Create one to begin. Exiting... \n");
        process::exit(1);
    }

    // Check for existing empty toml file (0 bytes in size)
    let is_empty = fs::metadata(&config_filename)
        .expect("Failed to read yeti.toml metadata")
        .len()
        == 0;

    if is_empty {
        ServerConfig::set_default_toml(&config_filename);
        println!("📝 Set default yeti.toml key-value pairs. Update the values and re-run yeti. Exiting... \n");
        process::exit(1);
    }

    // Read toml for configuration values
    let ServerConfig {
        port,
        // input_file,
        watch_dir,
        ..
    } = ServerConfig::read_toml(&config_filename);

    let port_addr: SocketAddr = format!("127.0.0.1:{port}")
        .parse()
        .expect("Failed to parse port address.");

    // Initialise Server Handler instance
    let server_handler = ServerHandler {};

    // Initialise shared file watcher & channel receiver
    let (_watcher, shared_rx) = WatchHandler::watcher(&watch_dir);

    // Setup listener and app for web server router
    let listener = TcpListener::bind(port_addr)
        .await
        .expect(format!("Failed to bind listener to address {port}").as_str());

    // Pass file watcher receiver to Web Socket route handler
    let app = Router::new()
        .route(
            "/ws",
            get(move |ws, connect_info| server_handler.ws_handler(ws, connect_info, shared_rx)),
        )
        .route_service("/client", ServeFile::new("client/client.js"));

    println!("✨ WebSockets Server active... \n");
    println!("🏠 Host Address: ");
    println!("   IP: {port_addr}");
    println!("   Socket: ws://localhost:{port}/ws \n");

    // Start the web server
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("Could not server web server");
}

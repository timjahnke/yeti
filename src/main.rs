use axum::routing::get;
use axum::Router;
use std::process::{self, Command};
use std::{env, net::SocketAddr};
use tokio::net::TcpListener;
use tower_http::services::ServeFile;

mod config;
mod server;
mod watcher;

use crate::config::{Config, ServerConfig};
use crate::server::ServerHandler;
use crate::watcher::WatchHandler;

#[tokio::main]
async fn main() {
    println!("🧊 Yeti v{}", env!("CARGO_PKG_VERSION"));

    // Check sass is installed in global path on the current system
    match Command::new("sass").arg("--version").output() {
        Ok(res) => {
            let formatted_output = res.stdout;
            println!("    Sass installed! Version {:?}", formatted_output);
        }
        Err(e) => {
            eprintln!("    Sass executable not found. \nError: {:?}", e);
            process::exit(1);
        }
    }

    let invoked_dir = env::current_dir().expect("Failed to get current directory");
    // println!("The current directory is {} \n", current_dir.display());

    // Check yeti is invoked in correct directory
    let config_filepath = format!("{}/yeti.toml", invoked_dir.display());

    // Get or create default config
    let server_config = ServerConfig::read_or_create_toml(&config_filepath);
    let Config {
        port,
        // input_file,
        watch_dir,
        ..
    } = server_config.get_config();

    let port_addr: SocketAddr = format!("127.0.0.1:{port}")
        .parse()
        .expect("Failed to parse port address.");

    // Initialise shared file watcher & channel receiver
    let (_watcher, shared_rx) = WatchHandler::new(watch_dir);
    // Initialise Server Handler instance & hashmap for active connections
    let server_handler = ServerHandler::new().await;

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

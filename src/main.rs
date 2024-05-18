use axum::routing::get;
use axum::Router;
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
    let current_dir = env::current_dir().expect("Failed to get current directory");
    // println!("The current directory is {} \n", current_dir.display());

    // Get config
    let config_filepath = format!("{}/yeti.toml", current_dir.display());
    let server_config = ServerConfig::access_toml(&config_filepath);
    let Config {
        port,
        input_file,
        watch_dir,
        ..
    } = server_config.get_config();

    let port_addr: SocketAddr = format!("127.0.0.1:{port}")
        .parse()
        .expect("Failed to parse port address.");

    println!("🧊 Yeti v{}", env!("CARGO_PKG_VERSION"));
    // Initialise shared file watcher & channel receiver
    let (watcher, shared_rx) = WatchHandler::new(watch_dir);
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
    println!("   Socket: ws://localhost:{port} \n");

    // Start the web server
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("Could not server web server");
}

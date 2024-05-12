use axum::routing::get;
use axum::{Router, ServiceExt};
use futures::executor;
use std::path::Path;
use std::{env, net::SocketAddr};
use tokio::net::TcpListener;
use tower_http::services::ServeFile;

mod config;
mod server;
mod sockets;
mod watcher;

use crate::config::{Config, ServerConfig};
use crate::server::ServerHandler;
use crate::watcher::WatchHandler;

#[tokio::main]
async fn main() {
    // Check invoked directory
    let current_dir = env::current_dir().expect("Failed to get current directory");
    println!("The current directory is {}", current_dir.display());

    // Get config
    let config_filepath = format!("{}/yeti.toml", current_dir.display());
    let server_config = ServerConfig::access_toml(&config_filepath);
    let Config {
        port,
        input_file,
        watch_dir,
        ..
    } = server_config.get_config();

    // Setup Web Server to listen on localhost port 8000
    let addr: &str = "127.0.0.1";
    let host_port_addr: SocketAddr = format!("{addr}:{port}")
        .parse()
        .expect("Failed to parse socket address.");

    // Initialise Server Handler instance & hashmap for active connections
    let server_handler = ServerHandler::new().await;

    // Initialise shared file watcher
    let watch_handler = WatchHandler::new().await;

    // Setup listener and app for web server router
    let listener = TcpListener::bind(host_port_addr)
        .await
        .expect(format!("Failed to bind listener to address {port}").as_str());

    // Pass file watcher to Web Socket route handler
    let app = Router::new()
        .route(
            "/ws",
            get(move |ws, connect_info| {
                server_handler
                    .clone()
                    .ws_handler(ws, connect_info, watch_handler.watcher)
            }),
        )
        .route_service("/client", ServeFile::new("client/client.js"));

    println!("   Yeti v{}", env!("CARGO_PKG_VERSION"));
    println!("🔌 WebSockets Server running... \n");
    println!("🏠 Host Address: ");
    println!("   IP: {host_port_addr}");
    println!("   Socket: ws://localhost:{port} \n");

    // Start the web server
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

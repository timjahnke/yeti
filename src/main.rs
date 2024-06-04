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

#[tokio::main]
async fn main() {
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

    let config_filepath = match std::env::current_dir() {
        Ok(path) => format!("{}/yeti.json", path.display()),
        Err(e) => {
            eprintln!("🚨 Failed to get current directory. Error: {:?}", e);
            process::exit(1);
        }
    };

    // Gracefully exit if no yeti.json file is found
    if !Path::new(&config_filepath).exists() {
        eprintln!("🚨 No yeti.json file found in the current directory. Create one to begin. Exiting... \n");
        process::exit(1);
    }

    // Check for existing empty json file (0 bytes in size)
    let is_json_empty = fs::metadata(&config_filepath)
        .expect("Failed to read yeti.json metadata")
        .len()
        == 0;

    if is_json_empty {
        ServerConfig::set_default_json_values(&config_filepath);
        println!("📝 Set default yeti.json key-value pairs. Update the values and re-run yeti. Exiting... \n");
        process::exit(1);
    }

    // Read yeti.json for configuration values
    let ServerConfig {
        port, style_tag_id, ..
    } = ServerConfig::read_json(&config_filepath);

    // Overwrite client.js file with style tag id and port
    ServerConfig::set_client_values(port, &style_tag_id);

    // Initialise Server Handler instance
    let server_handler = ServerHandler::new();

    // Create port address and parse
    let port_addr: SocketAddr = format!("127.0.0.1:{port}")
        .parse()
        .expect("Failed to parse port address.");

    // Setup listener and app for web server router
    let listener = match TcpListener::bind(port_addr).await {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("🚨 Port:{port} is in use. Try another port. \nError: {e}",);
            process::exit(1);
        }
    };

    // Pass file watcher receiver to Web Socket route handler
    let app = Router::new()
        .route(
            "/ws",
            get(move |ws, connect_info| {
                server_handler.clone().ws_handler(
                    ws,
                    connect_info,
                    server_handler.connections.clone(),
                )
            }),
        )
        .route_service(
            "/client",
            ServeFile::new(format!(
                "{}/yeti_client/client.js",
                std::env::current_exe()
                    .expect("Couldn't get current exe path")
                    .parent()
                    .expect("Couldn't get parent path")
                    .display()
            )),
        );
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

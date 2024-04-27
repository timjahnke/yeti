use owo_colors::OwoColorize;
use std::net::SocketAddr;
use tokio::net::TcpListener;

// Custom modules
mod server;
mod sockets;
mod watcher;

use crate::server::handle_connection;

// invoke: cargo run ./scss/

#[tokio::main]
async fn main() {
    // Setup server to listen on localhost port 8000
    let addr: &str = "127.0.0.1";
    let port: u16 = 8080;

    // Satisfy SocketAddr type. E.g. {}:{}""
    let port_addr: SocketAddr = format!("{}:{}", addr, &port)
        .parse()
        .expect("Failed to parse socket address.");

    // Setup event loop and TCP listener for connections
    let listener = TcpListener::bind(&port_addr)
        .await
        .expect(format!("Failed to bind listener to address {port_addr}").as_str());

    let socket_addr = format!("ws://localhost:{:?}", &port);

    println!();
    println!("{}", "🦀 Rust WebSockets v1.0".truecolor(251, 146, 60));
    println!("🔌 WebSockets Server running... \n");

    println!("🏠 Host Address: ");
    println!("   {} {}", "IP:".green(), &port_addr.green().underline());
    println!(
        "{}",
        format!(
            "   {} {} \n",
            "Socket:".green(),
            socket_addr.green().underline()
        )
    );

    // Setup socket server to listen for incoming connections
    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, port_addr));
    }
}

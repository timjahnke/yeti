use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
    process,
};

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct ServerConfig {
    pub watch_dir: String,
    pub input_file_path: String,
    pub output_file_path: String,
    pub style_tag_id: String,
    pub port: u16,
    pub stop_on_error: bool,
    pub experimental: bool,
}

impl ServerConfig {
    // Creates a new instance of the `ServerConfig` and stores parameter values within it.
    pub fn new(config: ServerConfig) -> Self {
        ServerConfig {
            watch_dir: config.watch_dir,
            input_file_path: config.input_file_path,
            output_file_path: config.output_file_path,
            style_tag_id: config.style_tag_id,
            port: config.port,
            stop_on_error: config.stop_on_error,
            experimental: config.experimental,
        }
    }

    /// Overwrites empty JSON file with default key-value pairs.
    pub fn set_default_json_values<P: AsRef<Path>>(file_path: P) {
        println!("Updating empty yeti.json file");
        // Overwrite file with some default content

        let json_str = r#"{
                "watch_dir": "scss_folder",
                "input_file_path": "scss/main.scss",
                "output_file_path": "dist/main.css",
                "port": 8080,
                "style_tag_id": "css-id",
                "stop_on_error": true,
                "experimental": false
            }"#;

        let json_value: ServerConfig =
            serde_json::from_str(json_str).expect("Failed to serialize JSON");

        // Open the file in write-only mode
        let mut file = File::create(file_path).expect("Failed to find/create file");

        file.write_all(
            serde_json::to_string_pretty(&json_value)
                .expect("Failed to serialize JSON")
                .as_bytes(),
        )
        .expect("Failed to write to file");
    }

    /// Read the JSON file and return the parsed JSON value.
    pub fn read_json<P: AsRef<Path>>(file_path: P) -> Self {
        // Unwrap safe as file existence checked prior
        let mut file = File::open(file_path).unwrap();

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Failed to read file contents");

        // Parse the JSON contents
        match serde_json::from_str(&contents) {
            Ok(config) => config,
            Err(err) => {
                eprintln!("Failed to parse JSON content. Error: {}", err);
                process::exit(1);
            }
        }
    }

    ///
    pub fn serve_javascript_string(port: u16, style_tag_id: &str) -> String {
        format!(
            r#"
                // Setup websocket connection for live reloading
                const socket = new WebSocket("ws://localhost:"+{port}+"/ws");
                console.info("Yeti client loaded");
                socket.onopen = function (event){{ 
                    console.info("Yeti connection opened");
                }};

                // Listen for messages from the Rust websocket server
                socket.onmessage = function (event) {{
                    const message = event.data;
                    console.info("Received message from Yeti server: " + message);

                    switch (message) {{
                        case "reload":
                        console.info("Reloading css");
                        const styleElement = document.getElementById("{style_tag_id}");

                        // Exit if style element not found
                        if (!styleElement) {{
                            console.error("Reload failed. Failed to find element with id: " + "{style_tag_id}");
                            return;
                        }}

                        const url = styleElement.getAttribute("href");

                        // Convert timestamp from milliseconds to seconds to mimic PHP time()
                        const timestampAsSeconds = Math.floor(new Date().getTime() / 1000);
                        // Add URL query to cache bust
                        const url_query = url+"?ver="+timestampAsSeconds;
                        // Set new URL to automatically fetch new css and bust cache
                        styleElement.setAttribute("href", url_query);
                        break;
                        default:
                        break;
                    }}
                }};

                socket.onerror = function (error) {{
                console.error("Yeti error: ", error);
                }};

                socket.onclose = function (event) {{
                console.log("Yeti Connection closed");
                }};

                addEventListener("beforeunload", (event) => {{
                socket.close();
                }});
        "#
        )
    }
}

use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Debug)]
pub struct ServerConfig {
    pub root_dir: String,
    pub project_dir: String,
    pub watch_dir: String,
    pub input_file: String,
    pub output_file: String,
    pub port: u16,
    pub style_id: String,
    pub experimental: bool,
}

impl ServerConfig {
    /// Overwrites empty TOML file with default key-value pairs.
    pub fn set_default_toml(file_path: &str) {
        println!("Updating empty Yeti toml file");
        // Overwrite file with some default content
        let default_content = r#"
            root_dir = "current"
            project_dir = "theme"
            watch_dir = "scss-folder"
            input_file = "main.scss"
            output_file = "main.css"
            port = 8080
            style_id = "sage-css"
            experimental = false
        "#;

        // Write the default content to the file
        fs::write(file_path, default_content).expect("Failed to update Yeti toml file");
    }

    /// Read the TOML file and return the parsed TOML value.
    pub fn read_toml(file_path: &str) -> Self {
        // Result type guaranteed due to prior checks
        let toml_content: String = fs::read_to_string(file_path).unwrap();

        // Parse the TOML content and return it as a TOML value
        let server_config: Self = match toml::from_str(&toml_content) {
            Ok(value) => value,
            Err(err) => {
                eprintln!("Error: {}", err);
                panic!("Failed to parse TOML content");
            }
        };
        server_config
    }
}

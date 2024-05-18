use serde::Deserialize;
use std::fs;
use std::io::Write;

#[derive(Deserialize, Debug)]
pub struct ServerConfig {
    config: Config,
}

#[derive(Deserialize, Debug)]
pub struct Config {
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
    /// Read the TOML file and return the parsed TOML value.
    /// Creates a TOML file if it doesn't exist.
    pub fn access_toml(file_path: &str) -> Self {
        let toml_content: String = match fs::read_to_string(file_path) {
            Ok(content) => content,
            Err(_) => {
                println!("Creating Yeti toml file");
                // If the file doesn't exist, create it with some default content
                let default_content = r#"
                [config]
                root_dir = "web"
                theme_dir = "sage-8"
                watch_path = "scss"
                input_path = "main.scss"
                output_path = "main.scss"
                port = 8080
            "#;

                // Write the default content to the file
                let mut file =
                    fs::File::create(file_path).expect("Failed to create Yeti toml file");
                file.write_all(default_content.as_bytes())
                    .expect("Failed to write to Yeti config file");

                // Return the default content as a TOML value
                String::from(default_content)
            }
        };

        // Parse the TOML content and return it as a TOML value
        let server_config: ServerConfig = match toml::from_str(&toml_content) {
            Ok(value) => value,
            Err(err) => {
                eprintln!("Error: {}", err);
                panic!("Failed to parse TOML content");
            }
        };
        server_config
    }

    pub fn get_config(&self) -> &Config {
        &self.config
    }
}

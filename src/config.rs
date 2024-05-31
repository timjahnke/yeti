use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{Read, Write},
    process,
};

#[derive(Deserialize, Serialize, Debug)]
pub struct ServerConfig {
    pub root_dir: String,
    pub project_dir: String,
    pub watch_dir: String,
    pub input_file_path: String,
    pub output_file_path: String,
    pub style_tag_id: String,
    pub port: u16,
    pub experimental: bool,
}

impl ServerConfig {
    /// Overwrites empty JSON file with default key-value pairs.
    pub fn set_default_json_values(file_path: &str) {
        println!("Updating empty yeti.json file");
        // Overwrite file with some default content

        let json_str = r#"{
                "root_dir": "current",
                "project_dir": "theme_name",
                "watch_dir": "scss_folder",
                "input_file_path": "scss/main.scss",
                "output_file_path": "dist/main.css",
                "port": 8080,
                "style_tag_id": "sage/css-css",
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
    pub fn read_json(file_path: &str) -> Self {
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

    pub fn set_client_values(port: u16, style_tag_id: &str) {
        let file_path = "client/client.js";

        // Read the contents of the file into a string
        let mut contents = fs::read_to_string(&file_path).expect("Failed to read js file.");

        // Lines to change in client.js file
        let updated_port = format!("const port = {port};");
        let updated_style_tag = format!("const style_tag_id = \"{style_tag_id}\";");

        let new_contents = contents
            .lines()
            .enumerate()
            .map(|(i, line)| match i {
                0 => updated_port.as_str(),
                1 => updated_style_tag.as_str(),
                _ => line,
            })
            .collect::<Vec<_>>()
            .join("\n");

        contents = new_contents;

        // Write the modified contents back to the file
        let mut file = fs::File::create(&file_path).expect("Failed to find/create file.");
        file.write_all(contents.as_bytes())
            .expect("Failed to write to file.");

        println!("📝 client.js updated successfully.");
    }
}

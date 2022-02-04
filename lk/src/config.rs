use serde::{Deserialize, Serialize};

use std::{
    fs::{self, OpenOptions},
    io::{BufWriter, Write},
    path::PathBuf,
};

#[derive(Serialize, Deserialize)]
pub struct Config {
    /// The default mode: fuzzy or list
    pub default_mode: String,
}

pub struct ConfigFile {
    pub config: Config,
    lk_dir: String,
    file_name: String,
}

impl ConfigFile {
    pub fn new(lk_dir: &str, file_name: &str) -> Self {
        let path = PathBuf::from(format!("{}/{}", lk_dir, file_name));
        // Create a default config file if it doesn't exist
        if !path.exists() {
            log::info!("Creating config file at {}", path.display());
            fs::create_dir(&path.parent().expect("failed to get `.config` dir"))
                .unwrap_or_else(|_| panic!("failed to create {} directory", path.display()));
            match OpenOptions::new().write(true).create(true).open(&path) {
                Ok(file) => {
                    let mut buffered = BufWriter::new(file);
                    let default_config = Config {
                        default_mode: "list".to_string(),
                    };
                    let toml = toml::to_string(&default_config).unwrap();
                    write!(buffered, "{}", toml).expect("Failed to write to file");
                }
                Err(e) => log::error!("Unable to create default config file: {}", e),
            }
        } else {
            log::info!("Using config file at {}", path.display());
        }

        // Load the config file
        let config_string = std::fs::read_to_string(path).expect("Couldn't read config file");
        let config = toml::from_str::<Config>(&config_string).expect("Couldn't parse config file");
        Self {
            config,
            lk_dir: lk_dir.to_string(),
            file_name: file_name.to_string(),
        }
    }

    pub fn save(&self) {
        let path = format!("{}/{}", self.lk_dir, self.file_name);
        let toml = toml::to_string(&self.config).expect("Couldn't serialize config file");
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&path)
            .unwrap_or_else(|_| panic!("Couldn't open config file at {}", path));
        let mut buffered = BufWriter::new(file);
        write!(buffered, "{}", toml).expect("Couldn't write to config file");
    }
}

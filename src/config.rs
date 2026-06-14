use serde::Deserialize;
use std::path::Path;

/// Application configuration loaded from a JSON file.
#[derive(Debug, Deserialize)]
pub struct Config {
    pub database: Database,
    pub server: Server,
}

#[derive(Debug, Deserialize)]
pub struct Database {
    /// SQLite database URL, e.g. "sqlite://moosic.db?mode=rwc"
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Server {
    /// Listen host, e.g. "0.0.0.0" or "127.0.0.1"
    pub host: String,
    /// Listen port, e.g. 3000
    pub port: u16,
}

impl Config {
    /// Load configuration from a JSON file.
    ///
    /// The path defaults to `config.json` in the current directory,
    /// and can be overridden by the `MOOSIC_CONFIG` environment variable.
    pub fn load() -> Self {
        let path = std::env::var("MOOSIC_CONFIG")
            .unwrap_or_else(|_| "config.json".to_owned());

        let path = Path::new(&path);
        let content = std::fs::read_to_string(path).unwrap_or_else(|e| {
            panic!("Failed to read config file {:?}: {}", path, e);
        });

        serde_json::from_str(&content).unwrap_or_else(|e| {
            panic!("Failed to parse config file {:?}: {}", path, e);
        })
    }
}

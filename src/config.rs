use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database: Database,
    #[serde(default)]
    pub cache: Cache,
    pub server: Server,
    #[serde(default)]
    pub frontend: Frontend,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum Database {
    #[serde(rename = "sqlite")]
    Sqlite { url: String },
}

#[derive(Debug, Clone, Deserialize)]
pub struct Server {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Frontend {
    /// Path to the built frontend directory containing static files and index.html.
    #[serde(default = "default_frontend_path")]
    pub path: String,
}

impl Default for Frontend {
    fn default() -> Self {
        Self {
            path: default_frontend_path(),
        }
    }
}

fn default_frontend_path() -> String {
    "web/dist".to_owned()
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum Cache {
    #[serde(rename = "moka")]
    Moka,
    #[serde(rename = "dashmap")]
    DashMap,
    #[serde(rename = "redis")]
    Redis {
        url: String,
    },
}

impl Default for Cache {
    fn default() -> Self {
        Self::Moka
    }
}

impl Config {
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

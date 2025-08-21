use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use parking_lot::RwLock;
use crate::generator::TerrainConfig;
use notify::{RecommendedWatcher, RecursiveMode, Watcher, Config as NotifyConfig};
use toml;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub terrain_config: TerrainConfig,
    pub tick_ms: f64,
    pub tile_growth_tick: u32,
    pub city_growth_tick: u32,
    pub capital_growth_tick: u32,
    pub city_visibility_radius: usize,
    pub tile_visibility_radius: usize,
    pub fow_mountains: bool,
    pub fow_swamps: bool,
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    pub fn default() -> Self {
        Self {
            terrain_config: TerrainConfig {
                mountain_density: 0.12,
                desert_density: 0.15,
                swamp_density: 0.08,
                city_density: 0.04,
                clustering_factor: 0.7,
                map_width: 20,
                map_height: 20,
            },
            tick_ms: 500.0,
            tile_growth_tick: 25,
            city_growth_tick: 10,
            capital_growth_tick: 5,
            city_visibility_radius: 3,
            tile_visibility_radius: 1,
            fow_mountains: false,
            fow_swamps: false,
        }
    }
}

pub type SharedConfig = Arc<RwLock<Config>>;

pub fn create_shared_config(path: Option<impl AsRef<Path> + Clone>) -> SharedConfig {
    let config = if let Some(path) = path.clone() {
        Config::load(&path).unwrap_or_else(|e| {
            eprintln!("Failed to load config: {}, using default", e);
            Config::default()
        })
    } else {
        Config::default()
    };
    let shared_config = Arc::new(RwLock::new(config));

    // Set up hot reloading if path is provided
    if let Some(path) = path {
        let path = path.as_ref().canonicalize().unwrap_or_else(|e| {
            eprintln!("Failed to canonicalize path: {}, using as-is", e);
            path.as_ref().to_path_buf()
        });
        let config_clone = shared_config.clone();

        tokio::spawn(async move {
                                    let (tx, mut rx) = tokio::sync::mpsc::channel(1);

            let mut watcher = RecommendedWatcher::new(
                move |res| {
                    let _ = tx.blocking_send(res);
                },
                NotifyConfig::default(),
            ).unwrap();

            // Watch the directory containing the config file
            let watch_path = if let Some(parent) = path.parent() {
                if parent.as_os_str().is_empty() {
                    Path::new(".")
                } else {
                    parent
                }
            } else {
                Path::new(".")
            };
            watcher.watch(watch_path, RecursiveMode::NonRecursive).unwrap();


            while let Some(res) = rx.recv().await {
                match res {
                    Ok(event) => {
                                                if event.paths.iter().any(|p| p.canonicalize().unwrap() == path.canonicalize().unwrap()) {
                            match Config::load(&path) {
                                Ok(new_config) => {
                                    *config_clone.write() = new_config;
                                }
                                Err(e) => {
                                    eprintln!("Failed to reload config: {}", e);
                                }
                            }
                        }
                    }
                    Err(e) => eprintln!("Watch error: {}", e),
                }
            }
        });
    }

    shared_config
}

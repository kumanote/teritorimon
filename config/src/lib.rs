pub type Result<T> = anyhow::Result<T>;

pub mod configs;
mod threshold;
mod toml;

pub use threshold::MissedBlockThreshold;

use configs::{ApplicationConfig, FromEnv};
use once_cell::sync::OnceCell;
use std::path::Path;
use std::sync::Arc;

static GLOBAL_APP_CONFIG: OnceCell<Arc<ApplicationConfig>> = OnceCell::new();

pub fn load_app_config<P: AsRef<Path>>(config_toml_path: Option<P>) -> Result<ApplicationConfig> {
    match config_toml_path {
        Some(path) => ApplicationConfig::load_from_file(path),
        None => ApplicationConfig::from_env(),
    }
}

pub fn app_config() -> &'static ApplicationConfig {
    GLOBAL_APP_CONFIG
        .get()
        .expect("app config must be initialized...")
}

pub fn set_app_config(app_config: Arc<ApplicationConfig>) {
    if GLOBAL_APP_CONFIG.set(app_config).is_err() {
        eprintln!("Global logger has already been set");
    }
}

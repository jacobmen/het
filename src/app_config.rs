use anyhow::Result;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct AppConfig {
    pub db_path: PathBuf,
}

pub fn get_app_config(config_file: &str) -> Result<AppConfig> {
    Ok(toml::from_str::<AppConfig>(config_file)?)
}

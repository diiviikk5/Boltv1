use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::models::PriorityClass;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub steam_root: Option<PathBuf>,
    pub legendary_path: Option<PathBuf>,
    pub gogdl_path: Option<PathBuf>,
    pub amazon_path: Option<PathBuf>,
    pub default_priority: PriorityClass,
    pub default_kill_list: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            steam_root: None,
            legendary_path: None,
            gogdl_path: None,
            amazon_path: None,
            default_priority: PriorityClass::Normal,
            default_kill_list: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub root_dir: PathBuf,
    pub config_file: PathBuf,
    pub db_file: PathBuf,
    pub export_dir: PathBuf,
}

pub fn resolve_paths() -> Result<AppPaths> {
    let project_dirs = ProjectDirs::from("com", "bolt", "bolt")
        .context("failed to resolve application directories")?;
    let root_dir = project_dirs.data_local_dir().to_path_buf();
    let config_dir = project_dirs.config_local_dir().to_path_buf();
    fs::create_dir_all(&root_dir)?;
    fs::create_dir_all(&config_dir)?;
    let export_dir = root_dir.join("exports");
    fs::create_dir_all(&export_dir)?;
    Ok(AppPaths {
        root_dir: root_dir.clone(),
        config_file: config_dir.join("config.toml"),
        db_file: root_dir.join("library.sqlite3"),
        export_dir,
    })
}

pub fn load(paths: &AppPaths) -> Result<AppConfig> {
    if !paths.config_file.exists() {
        let config = AppConfig::default();
        save(paths, &config)?;
        return Ok(config);
    }
    let raw = fs::read_to_string(&paths.config_file)?;
    Ok(toml::from_str(&raw)?)
}

pub fn save(paths: &AppPaths, config: &AppConfig) -> Result<()> {
    let raw = toml::to_string_pretty(config)?;
    fs::write(&paths.config_file, raw)?;
    Ok(())
}

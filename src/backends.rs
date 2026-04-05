use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use chrono::Utc;
use regex::Regex;
use serde_json::Value;
use which::which;

use crate::config::AppConfig;
use crate::models::{Game, GameSource, LaunchProfile, LaunchTarget, LauncherMetadata, stable_id};

pub trait Backend {
    fn detect(&self, config: &AppConfig) -> bool;
    fn import_games(&self, config: &AppConfig) -> Result<Vec<Game>>;
    fn resolve_launch_target(&self, game: &Game, config: &AppConfig) -> Result<Option<LaunchTarget>>;
    fn launch(&self, game: &Game, config: &AppConfig) -> Result<Option<LaunchTarget>> {
        self.resolve_launch_target(game, config)
    }
}

pub struct SteamBackend;
pub struct LegendaryBackend;
pub struct GogBackend;
pub struct AmazonBackend;

impl Backend for SteamBackend {
    fn detect(&self, config: &AppConfig) -> bool {
        steam_root(config).is_some()
    }

    fn import_games(&self, config: &AppConfig) -> Result<Vec<Game>> {
        let steam_root = if let Some(path) = steam_root(config) {
            path
        } else {
            return Ok(Vec::new());
        };
        let steam_exe = steam_root.join("steam.exe");
        let mut libraries = discover_steam_libraries(&steam_root)?;
        if !libraries.iter().any(|path| path == &steam_root) {
            libraries.push(steam_root.clone());
        }
        let mut games = Vec::new();
        for library in libraries {
            let steamapps = library.join("steamapps");
            if !steamapps.exists() {
                continue;
            }
            for entry in std::fs::read_dir(&steamapps)? {
                let entry = entry?;
                let path = entry.path();
                let name = path.file_name().and_then(|v| v.to_str()).unwrap_or_default();
                if !name.starts_with("appmanifest_") || path.extension().and_then(|v| v.to_str()) != Some("acf") {
                    continue;
                }
                if let Some((app_id, title)) = parse_steam_manifest(&path)? {
                    let now = Utc::now();
                    games.push(Game {
                        id: stable_id(GameSource::Steam, &steam_exe, Some(&app_id)),
                        name: title,
                        source: GameSource::Steam,
                        executable: steam_exe.clone(),
                        working_dir: steam_root.clone(),
                        launch_args: vec!["-applaunch".into(), app_id.clone()],
                        env_vars: HashMap::new(),
                        launcher_metadata: LauncherMetadata {
                            launcher_id: Some(app_id),
                            launcher_path: Some(steam_exe.clone()),
                            notes: HashMap::from([("store".into(), "steam".into())]),
                        },
                        profile: LaunchProfile::default(),
                        last_played: None,
                        play_count: 0,
                        created_at: now,
                        updated_at: now,
                    });
                }
            }
        }
        Ok(games)
    }

    fn resolve_launch_target(&self, game: &Game, _config: &AppConfig) -> Result<Option<LaunchTarget>> {
        Ok(Some(passthrough_target(game)))
    }
}

impl Backend for LegendaryBackend {
    fn detect(&self, config: &AppConfig) -> bool {
        resolve_tool(config.legendary_path.as_deref(), &["legendary.exe", "legendary"]).is_some()
    }

    fn import_games(&self, config: &AppConfig) -> Result<Vec<Game>> {
        let Some(tool) = resolve_tool(config.legendary_path.as_deref(), &["legendary.exe", "legendary"]) else {
            return Ok(Vec::new());
        };
        let output = Command::new(&tool)
            .args(["list-installed", "--json"])
            .output()
            .with_context(|| format!("failed to execute {}", tool.display()))?;
        if !output.status.success() {
            return Ok(Vec::new());
        }
        parse_legendary_games(&tool, &String::from_utf8_lossy(&output.stdout))
    }

    fn resolve_launch_target(&self, game: &Game, _config: &AppConfig) -> Result<Option<LaunchTarget>> {
        Ok(Some(passthrough_target(game)))
    }
}

impl Backend for GogBackend {
    fn detect(&self, config: &AppConfig) -> bool {
        resolve_tool(config.gogdl_path.as_deref(), &["gogdl.exe", "gogdl"]).is_some()
    }

    fn import_games(&self, config: &AppConfig) -> Result<Vec<Game>> {
        let Some(tool) = resolve_tool(config.gogdl_path.as_deref(), &["gogdl.exe", "gogdl"]) else {
            return Ok(Vec::new());
        };
        let output = Command::new(&tool)
            .args(["list-installed", "--json"])
            .output()
            .with_context(|| format!("failed to execute {}", tool.display()))?;
        if !output.status.success() {
            return Ok(Vec::new());
        }
        parse_gog_games(&tool, &String::from_utf8_lossy(&output.stdout))
    }

    fn resolve_launch_target(&self, game: &Game, _config: &AppConfig) -> Result<Option<LaunchTarget>> {
        Ok(Some(passthrough_target(game)))
    }
}

impl Backend for AmazonBackend {
    fn detect(&self, config: &AppConfig) -> bool {
        resolve_tool(config.amazon_path.as_deref(), &["amazon-games.exe", "amazon-games"]).is_some()
    }

    fn import_games(&self, config: &AppConfig) -> Result<Vec<Game>> {
        let Some(tool) = resolve_tool(config.amazon_path.as_deref(), &["amazon-games.exe", "amazon-games"]) else {
            return Ok(Vec::new());
        };
        let output = Command::new(&tool)
            .args(["list-installed", "--json"])
            .output()
            .with_context(|| format!("failed to execute {}", tool.display()))?;
        if !output.status.success() {
            return Ok(Vec::new());
        }
        parse_amazon_games(&tool, &String::from_utf8_lossy(&output.stdout))
    }

    fn resolve_launch_target(&self, game: &Game, _config: &AppConfig) -> Result<Option<LaunchTarget>> {
        Ok(Some(passthrough_target(game)))
    }
}

pub fn backend_for_source(source: &GameSource) -> Box<dyn Backend> {
    match source {
        GameSource::Local => Box::new(LocalBackend),
        GameSource::Steam => Box::new(SteamBackend),
        GameSource::Epic => Box::new(LegendaryBackend),
        GameSource::Gog => Box::new(GogBackend),
        GameSource::Amazon => Box::new(AmazonBackend),
    }
}

pub fn import_from_all(config: &AppConfig) -> Result<Vec<Game>> {
    let backends: Vec<Box<dyn Backend>> = vec![
        Box::new(SteamBackend),
        Box::new(LegendaryBackend),
        Box::new(GogBackend),
        Box::new(AmazonBackend),
    ];
    let mut all = Vec::new();
    for backend in backends {
        if backend.detect(config) {
            all.extend(backend.import_games(config)?);
        }
    }
    Ok(all)
}

struct LocalBackend;

impl Backend for LocalBackend {
    fn detect(&self, _config: &AppConfig) -> bool {
        true
    }

    fn import_games(&self, _config: &AppConfig) -> Result<Vec<Game>> {
        Ok(Vec::new())
    }

    fn resolve_launch_target(&self, game: &Game, _config: &AppConfig) -> Result<Option<LaunchTarget>> {
        Ok(Some(passthrough_target(game)))
    }
}

fn passthrough_target(game: &Game) -> LaunchTarget {
    LaunchTarget {
        executable: game.executable.clone(),
        args: game.launch_args.clone(),
        working_dir: game.working_dir.clone(),
        env: game.env_vars.clone(),
        source: game.source.clone(),
        metadata: game.launcher_metadata.clone(),
    }
}

fn steam_root(config: &AppConfig) -> Option<PathBuf> {
    if let Some(path) = config.steam_root.clone() {
        return Some(path);
    }
    [
        PathBuf::from(r"C:\Program Files (x86)\Steam"),
        PathBuf::from(r"C:\Program Files\Steam"),
    ]
    .into_iter()
    .find(|path| path.join("steam.exe").exists())
}

fn discover_steam_libraries(root: &Path) -> Result<Vec<PathBuf>> {
    let library_file = root.join("steamapps").join("libraryfolders.vdf");
    if !library_file.exists() {
        return Ok(vec![root.to_path_buf()]);
    }
    let raw = std::fs::read_to_string(library_file)?;
    let regex = Regex::new(r#""path"\s*"([^"]+)""#)?;
    let mut libraries = Vec::new();
    for capture in regex.captures_iter(&raw) {
        libraries.push(PathBuf::from(capture[1].replace("\\\\", "\\")));
    }
    if libraries.is_empty() {
        libraries.push(root.to_path_buf());
    }
    Ok(libraries)
}

fn parse_steam_manifest(path: &Path) -> Result<Option<(String, String)>> {
    let raw = std::fs::read_to_string(path)?;
    let app_id = Regex::new(r#""appid"\s*"([^"]+)""#)?
        .captures(&raw)
        .map(|caps| caps[1].to_string());
    let name = Regex::new(r#""name"\s*"([^"]+)""#)?
        .captures(&raw)
        .map(|caps| caps[1].to_string());
    Ok(match (app_id, name) {
        (Some(app_id), Some(name)) => Some((app_id, name)),
        _ => None,
    })
}

fn resolve_tool(config_path: Option<&Path>, candidates: &[&str]) -> Option<PathBuf> {
    if let Some(path) = config_path.filter(|path| path.exists()) {
        return Some(path.to_path_buf());
    }
    for candidate in candidates {
        if let Ok(found) = which(candidate) {
            return Some(found);
        }
    }
    None
}

fn parse_legendary_games(tool: &Path, raw: &str) -> Result<Vec<Game>> {
    let parsed: Value = serde_json::from_str(raw)?;
    let entries = collect_entries(&parsed);
    let mut games = Vec::new();
    for entry in entries {
        let title = string_field(entry, &["title", "app_title", "name"]);
        let app_name = string_field(entry, &["app_name", "appName", "id"]);
        if let (Some(name), Some(app_id)) = (title, app_name) {
            let now = Utc::now();
            games.push(Game {
                id: stable_id(GameSource::Epic, tool, Some(&app_id)),
                name,
                source: GameSource::Epic,
                executable: tool.to_path_buf(),
                working_dir: tool.parent().unwrap_or_else(|| Path::new(".")).to_path_buf(),
                launch_args: vec!["launch".into(), app_id.clone()],
                env_vars: HashMap::new(),
                launcher_metadata: LauncherMetadata {
                    launcher_id: Some(app_id),
                    launcher_path: Some(tool.to_path_buf()),
                    notes: HashMap::new(),
                },
                profile: LaunchProfile::default(),
                last_played: None,
                play_count: 0,
                created_at: now,
                updated_at: now,
            });
        }
    }
    Ok(games)
}

fn parse_gog_games(tool: &Path, raw: &str) -> Result<Vec<Game>> {
    let parsed: Value = serde_json::from_str(raw)?;
    let entries = collect_entries(&parsed);
    let mut games = Vec::new();
    for entry in entries {
        let title = string_field(entry, &["title", "name"]);
        let game_id = string_field(entry, &["id", "game_id", "gameId"]);
        if let (Some(name), Some(game_id)) = (title, game_id) {
            let now = Utc::now();
            games.push(Game {
                id: stable_id(GameSource::Gog, tool, Some(&game_id)),
                name,
                source: GameSource::Gog,
                executable: tool.to_path_buf(),
                working_dir: tool.parent().unwrap_or_else(|| Path::new(".")).to_path_buf(),
                launch_args: vec!["launch".into(), "--id".into(), game_id.clone()],
                env_vars: HashMap::new(),
                launcher_metadata: LauncherMetadata {
                    launcher_id: Some(game_id),
                    launcher_path: Some(tool.to_path_buf()),
                    notes: HashMap::new(),
                },
                profile: LaunchProfile::default(),
                last_played: None,
                play_count: 0,
                created_at: now,
                updated_at: now,
            });
        }
    }
    Ok(games)
}

fn parse_amazon_games(tool: &Path, raw: &str) -> Result<Vec<Game>> {
    let parsed: Value = serde_json::from_str(raw)?;
    let entries = collect_entries(&parsed);
    let mut games = Vec::new();
    for entry in entries {
        let title = string_field(entry, &["title", "name"]);
        let game_id = string_field(entry, &["id", "game_id", "gameId"]);
        if let (Some(name), Some(game_id)) = (title, game_id) {
            let now = Utc::now();
            games.push(Game {
                id: stable_id(GameSource::Amazon, tool, Some(&game_id)),
                name,
                source: GameSource::Amazon,
                executable: tool.to_path_buf(),
                working_dir: tool.parent().unwrap_or_else(|| Path::new(".")).to_path_buf(),
                launch_args: vec!["launch".into(), game_id.clone()],
                env_vars: HashMap::new(),
                launcher_metadata: LauncherMetadata {
                    launcher_id: Some(game_id),
                    launcher_path: Some(tool.to_path_buf()),
                    notes: HashMap::new(),
                },
                profile: LaunchProfile::default(),
                last_played: None,
                play_count: 0,
                created_at: now,
                updated_at: now,
            });
        }
    }
    Ok(games)
}

fn collect_entries<'a>(value: &'a Value) -> Vec<&'a Value> {
    if let Some(array) = value.as_array() {
        return array.iter().collect();
    }
    if let Some(array) = value
        .get("games")
        .and_then(Value::as_array)
        .or_else(|| value.get("installed").and_then(Value::as_array))
        .or_else(|| value.get("data").and_then(Value::as_array))
    {
        return array.iter().collect();
    }
    Vec::new()
}

fn string_field(value: &Value, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(result) = value.get(*key).and_then(Value::as_str) {
            return Some(result.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use tempfile::tempdir;

    use super::{parse_gog_games, parse_legendary_games, parse_steam_manifest};

    #[test]
    fn parses_steam_manifest() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("appmanifest_123.acf");
        std::fs::write(
            &path,
            "\"AppState\"\n{\n\"appid\"\t\t\"123\"\n\"name\"\t\t\"Portal\"\n}\n",
        )
        .expect("write");
        let parsed = parse_steam_manifest(&path).expect("parse");
        assert_eq!(parsed, Some(("123".into(), "Portal".into())));
    }

    #[test]
    fn parses_legendary_json() {
        let raw = r#"[{"title":"Alan Wake 2","app_name":"AW2"}]"#;
        let games = parse_legendary_games(Path::new(r"C:\legendary.exe"), raw).expect("parse");
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].launch_args, vec!["launch", "AW2"]);
    }

    #[test]
    fn parses_gog_json() {
        let raw = r#"{"games":[{"title":"Cyberpunk 2077","id":"1423049311"}]}"#;
        let games = parse_gog_games(Path::new(r"C:\gogdl.exe"), raw).expect("parse");
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].launch_args, vec!["launch", "--id", "1423049311"]);
    }
}

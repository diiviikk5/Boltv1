use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

use crate::cli::PriorityArg;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GameSource {
    Local,
    Steam,
    Epic,
    Gog,
    Amazon,
}

impl Display for GameSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Local => "local",
            Self::Steam => "steam",
            Self::Epic => "epic",
            Self::Gog => "gog",
            Self::Amazon => "amazon",
        };
        f.write_str(value)
    }
}

impl std::str::FromStr for GameSource {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value.to_ascii_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "steam" => Ok(Self::Steam),
            "epic" => Ok(Self::Epic),
            "gog" => Ok(Self::Gog),
            "amazon" => Ok(Self::Amazon),
            _ => Err(anyhow!("unknown game source: {value}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PriorityClass {
    Idle,
    BelowNormal,
    Normal,
    AboveNormal,
    High,
    Realtime,
}

impl Default for PriorityClass {
    fn default() -> Self {
        Self::Normal
    }
}

impl From<PriorityArg> for PriorityClass {
    fn from(value: PriorityArg) -> Self {
        match value {
            PriorityArg::Idle => Self::Idle,
            PriorityArg::BelowNormal => Self::BelowNormal,
            PriorityArg::Normal => Self::Normal,
            PriorityArg::AboveNormal => Self::AboveNormal,
            PriorityArg::High => Self::High,
            PriorityArg::Realtime => Self::Realtime,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct LaunchProfile {
    pub priority: PriorityClass,
    pub affinity_mask: Option<u64>,
    pub env_overrides: HashMap<String, String>,
    pub kill_after_launch: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct LauncherMetadata {
    pub launcher_id: Option<String>,
    pub launcher_path: Option<PathBuf>,
    pub notes: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LaunchTarget {
    pub executable: PathBuf,
    pub args: Vec<String>,
    pub working_dir: PathBuf,
    pub env: HashMap<String, String>,
    pub source: GameSource,
    pub metadata: LauncherMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Game {
    pub id: String,
    pub name: String,
    pub source: GameSource,
    pub executable: PathBuf,
    pub working_dir: PathBuf,
    pub launch_args: Vec<String>,
    pub env_vars: HashMap<String, String>,
    pub launcher_metadata: LauncherMetadata,
    pub profile: LaunchProfile,
    pub last_played: Option<DateTime<Utc>>,
    pub play_count: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Game {
    pub fn local(name: String, executable: PathBuf) -> Self {
        let now = Utc::now();
        let working_dir = executable
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        Self {
            id: stable_id(GameSource::Local, &executable, None),
            name,
            source: GameSource::Local,
            executable,
            working_dir,
            launch_args: Vec::new(),
            env_vars: HashMap::new(),
            launcher_metadata: LauncherMetadata::default(),
            profile: LaunchProfile::default(),
            last_played: None,
            play_count: 0,
            created_at: now,
            updated_at: now,
        }
    }
}

pub fn stable_id(source: GameSource, executable: &Path, launcher_id: Option<&str>) -> String {
    let mut digest = Sha1::new();
    digest.update(source.to_string());
    digest.update(b"|");
    digest.update(executable.to_string_lossy().as_bytes());
    if let Some(value) = launcher_id {
        digest.update(b"|");
        digest.update(value.as_bytes());
    }
    format!("{:x}", digest.finalize())
}

pub fn parse_affinity_mask(value: &str) -> Result<u64> {
    let trimmed = value.trim();
    let parsed = if let Some(hex) = trimmed.strip_prefix("0x").or_else(|| trimmed.strip_prefix("0X")) {
        u64::from_str_radix(hex, 16)?
    } else {
        trimmed.parse()?
    };
    Ok(parsed)
}

pub fn display_path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

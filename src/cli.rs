use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

pub const BANNER: &str = r#"
 ____   ___   _  _____
| __ ) / _ \ | ||_   _|
|  _ \| | | || |  | |
| |_) | |_| || |__| |
|____/ \___/ |____|_|
"#;

#[derive(Debug, Parser)]
#[command(
    name = "bolt",
    version,
    about = "Windows-first zero-bloat game launcher",
    before_help = BANNER,
    after_help = "Examples:\n  bolt add\n  bolt add \"D:\\Games\\Game\\game.exe\" --name \"Game\"\n  bolt launch cyberpunk\n  bolt import all\n  bolt export \"Game Name\""
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(alias = "a")]
    Add(AddArgs),
    #[command(alias = "s")]
    Scan(ScanArgs),
    #[command(alias = "ls")]
    List(ListArgs),
    #[command(alias = "run")]
    Launch(LaunchArgs),
    #[command(alias = "sync")]
    Import(ImportArgs),
    #[command(alias = "cfg")]
    Config(ConfigArgs),
    #[command(alias = "x")]
    Export(ExportArgs),
}

#[derive(Debug, Args)]
pub struct AddArgs {
    pub path: Option<PathBuf>,
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long = "arg")]
    pub args: Vec<String>,
    #[arg(long = "env")]
    pub env: Vec<String>,
    #[arg(long)]
    pub priority: Option<PriorityArg>,
    #[arg(long)]
    pub affinity: Option<String>,
    #[arg(long = "kill")]
    pub kill_after_launch: Vec<String>,
}

#[derive(Debug, Args)]
pub struct ScanArgs {
    pub dir: PathBuf,
    #[arg(long)]
    pub flat: bool,
}

#[derive(Debug, Args)]
pub struct ListArgs {
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct LaunchArgs {
    pub query: String,
    #[arg(long)]
    pub exact: bool,
}

#[derive(Debug, Args)]
pub struct ImportArgs {
    pub source: ImportSource,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ImportSource {
    Steam,
    Epic,
    Gog,
    Amazon,
    All,
}

#[derive(Debug, Args)]
pub struct ConfigArgs {
    pub query: String,
    #[arg(long)]
    pub show: bool,
    #[arg(long)]
    pub priority: Option<PriorityArg>,
    #[arg(long)]
    pub affinity: Option<String>,
    #[arg(long = "kill")]
    pub kill_after_launch: Vec<String>,
    #[arg(long = "env")]
    pub env: Vec<String>,
    #[arg(long)]
    pub clear_env: bool,
    #[arg(long)]
    pub clear_kill_rules: bool,
}

#[derive(Debug, Args)]
pub struct ExportArgs {
    pub query: String,
    #[arg(long)]
    pub output: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum PriorityArg {
    Idle,
    BelowNormal,
    Normal,
    AboveNormal,
    High,
    Realtime,
}

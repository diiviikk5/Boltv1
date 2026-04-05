use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "bolt", version, about = "Windows-first zero-bloat game launcher")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Add(AddArgs),
    Scan(ScanArgs),
    List(ListArgs),
    Launch(LaunchArgs),
    Import(ImportArgs),
    Config(ConfigArgs),
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

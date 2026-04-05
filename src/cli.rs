use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

pub const MASCOT_COLOR: &str = concat!(
    "\x1b[38;5;45m⠀⠀⣀⣀⣀⣀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀\x1b[0m\n",
    "\x1b[38;5;81m⢠⣾⠟⠉⠉⠛⠿⢶⣤⣀⠀⣀⣀⣀⣀⣀⣀⡀⠀⠀⠀⠀⣀⣠⣴⠶⠶⢶⣦⡀\x1b[0m\n",
    "\x1b[38;5;117m⢸⡇⠀⠀⠀⠀⠀⠀⠈⠛⡛⠙⠉⡹⠙⠛⠋⠟⠛⠶⣶⡿⠋⠉⠀⠀⠀⠀⢹⣧\x1b[0m\n",
    "\x1b[38;5;123m⢸⡇⠀⠀⠀⠀⠀⠀⠀⢰⠁⡄⠀⡇⢸⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⡇\x1b[0m\n",
    "\x1b[38;5;159m⠘⠿⠀⠀⠀⠀⠀⠀⠀⠀⠉⠁⠀⠀⠀⠀⠈⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣾⠃\x1b[0m\n",
    "\x1b[38;5;153m⠀⠀⠀⢀⣠⣤⣤⣤⣄⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠿⠏⠀\x1b[0m\n",
    "\x1b[38;5;147m⠀⠀⣰⡟⠁⠀⠀⠙⣿⣿⡄⠀⠒⠀⠀⠀⠀⠀⢀⣤⡴⠛⠛⠻⣷⣦⡀⠀⠀⠀\x1b[0m\n",
    "\x1b[38;5;111m⠀⠀⣿⣇⠀⠀⢀⣰⣿⣿⣿⡄⠀⠀⠀⠀⠀⢀⣾⡏⠀⠀⠀⠀⢈⣿⣿⠀⠀⠀\x1b[0m\n",
    "\x1b[38;5;75m⠀⠀⢿⣿⣿⣿⣿⣿⣿⣿⣿⡧⠀⠀⠀⠀⠀⣼⣿⣧⣤⣤⣤⣴⣿⣿⣿⠀⠀⠀\x1b[0m\n",
    "\x1b[38;5;69m⢀⠂⠉⡻⢿⣿⣿⣿⣏⢹⣿⠇⠀⠀⠀⠀⠀⣿⣿⣿⣿⡟⢿⣿⣿⣿⠃⣀⠀⠀\x1b[0m\n",
    "\x1b[38;5;63m⠀⠉⠙⠿⣦⣍⡛⠛⠛⠉⠁⠀⠀⠀⠀⠀⠀⠈⠛⠿⠿⠷⢟⣛⣽⡧⠤⠔⠀⠀\x1b[0m\n",
    "\x1b[38;5;99m⠀⠀⠀⠀⠈⠙⠛⢿⣶⣶⣦⣤⡀⠀⣀⣀⣤⣀⣤⣤⣴⡾⠟⠛⠉⠀⠀⠀⠀⠀\x1b[0m\n",
    "\x1b[38;5;135m⠀⠀⠀⠀⠀⠀⠀⠘⣿⡆⠀⠙⠿⠿⠛⠉⣉⣽⡿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀\x1b[0m\n",
    "\x1b[38;5;171m⠀⠀⠀⠀⠀⠀⠀⠀⠸⣧⣠⣴⠶⢶⣤⣴⡿⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀\x1b[0m\n",
    "\x1b[38;5;207m⠀⠀⠀⠀⠀⠀⠀⠀⠀⠙⠛⠃⠀⠀⠛⠋⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀\x1b[0m\n",
    "\x1b[1;38;5;45mBOLT\x1b[0m \x1b[38;5;252mWindows-first zero-bloat game launcher\x1b[0m\n"
);

#[derive(Debug, Parser)]
#[command(
    name = "bolt",
    version,
    about = "Windows-first zero-bloat game launcher",
    before_help = MASCOT_COLOR,
    after_help = "Examples:\n  bolt add\n  bolt add \"D:\\Games\\Game\\game.exe\" --name \"Game\"\n  bolt launch cyberpunk\n  bolt import all\n  bolt status\n  bolt tune \"Game Name\" --mode safe\n  bolt export \"Game Name\""
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
    #[command(alias = "st")]
    Status,
    #[command(alias = "t")]
    Tune(TuneArgs),
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

#[derive(Debug, Args)]
pub struct TuneArgs {
    pub query: String,
    #[arg(long, value_enum, default_value_t = TuneModeArg::Safe)]
    pub mode: TuneModeArg,
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

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum TuneModeArg {
    Safe,
    Aggressive,
}

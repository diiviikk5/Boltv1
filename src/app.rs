use std::collections::HashMap;
use std::io::{self, Write};
use std::path::Path;
use std::time::Instant;

use anyhow::{Context, Result, anyhow};
use chrono::Utc;
use clap::Parser;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use rfd::FileDialog;
use walkdir::WalkDir;

use crate::backends::{
    AmazonBackend, Backend, GogBackend, LegendaryBackend, SteamBackend, backend_for_source, import_from_all,
};
use crate::cli::{
    AddArgs, Cli, Commands, ConfigArgs, ExportArgs, ImportArgs, ImportSource, LaunchArgs, ListArgs,
    MASCOT_COLOR, ScanArgs, TuneArgs, TuneModeArg,
};
use crate::config::{load as load_config, resolve_paths};
use crate::db::Database;
use crate::launcher;
use crate::models::{Game, GameSource, PriorityClass, display_path, parse_affinity_mask};

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let paths = resolve_paths()?;
    let _root_dir = &paths.root_dir;
    let config = load_config(&paths)?;
    let db = Database::open(&paths.db_file)?;
    match cli.command {
        Some(command) => execute_command(&db, &config, &paths.export_dir, command),
        None => run_shell(&db, &config, &paths.export_dir),
    }
}

fn execute_command(
    db: &Database,
    config: &crate::config::AppConfig,
    export_dir: &Path,
    command: Commands,
) -> Result<()> {
    match command {
        Commands::Add(args) => add_game(&db, args, &config.default_kill_list, config.default_priority.clone()),
        Commands::Scan(args) => scan_games(&db, args, &config.default_kill_list, config.default_priority.clone()),
        Commands::List(args) => list_games(&db, args),
        Commands::Launch(args) => launch_game(&db, &config, args),
        Commands::Import(args) => import_games(&db, &config, args),
        Commands::Config(args) => configure_game(&db, args),
        Commands::Export(args) => export_game(&db, &config, export_dir, args),
        Commands::Status => show_status(&db, &config),
        Commands::Tune(args) => tune_game(&db, args),
    }
}

fn run_shell(db: &Database, config: &crate::config::AppConfig, export_dir: &Path) -> Result<()> {
    println!("{MASCOT_COLOR}");
    println!("Interactive mode. Type `help` for commands, `exit` to quit.");
    loop {
        print!("bolt> ");
        io::stdout().flush()?;
        let mut line = String::new();
        match io::stdin().read_line(&mut line) {
            Ok(0) => {
                println!();
                return Ok(());
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {
                println!();
                return Ok(());
            }
            Err(error) => return Err(error.into()),
        }

        let input = line.trim();
        if input.is_empty() {
            continue;
        }
        if matches!(input, "exit" | "quit") {
            return Ok(());
        }
        if matches!(input, "help" | "?") {
            print_shell_help();
            continue;
        }

        let argv = match shell_argv(input) {
            Ok(argv) => argv,
            Err(error) => {
                eprintln!("error: {error}");
                continue;
            }
        };
        match Cli::try_parse_from(argv) {
            Ok(cli) => {
                if let Some(command) = cli.command {
                    if let Err(error) = execute_command(db, config, export_dir, command) {
                        eprintln!("error: {error:#}");
                    }
                } else {
                    print_shell_help();
                }
            }
            Err(error) => {
                let _ = error.print();
            }
        }
    }
}

fn print_shell_help() {
    println!("Commands: add | scan | list | launch | import | config | export | status | tune");
    println!("Aliases:  a   | s    | ls   | run    | sync   | cfg    | x      | st     | t");
    println!("Examples:");
    println!("  bolt> add");
    println!("  bolt> add \"D:\\Games\\Game\\game.exe\" --name \"Game\"");
    println!("  bolt> list");
    println!("  bolt> launch cyberpunk");
    println!("  bolt> status");
    println!("  bolt> tune \"Game Name\" --mode safe");
    println!("  bolt> export \"Game Name\"");
}

fn shell_argv(input: &str) -> Result<Vec<String>> {
    let mut parts = shlex::split(input).ok_or_else(|| anyhow!("invalid quotes in command"))?;
    if parts.is_empty() {
        return Ok(vec!["bolt".to_string()]);
    }
    if parts[0].eq_ignore_ascii_case("bolt") {
        parts.remove(0);
    }
    let mut argv = vec!["bolt".to_string()];
    argv.extend(parts);
    Ok(argv)
}

fn add_game(
    db: &Database,
    args: AddArgs,
    default_kill_list: &[String],
    default_priority: PriorityClass,
) -> Result<()> {
    let path = match args.path {
        Some(path) => path,
        None => FileDialog::new()
            .add_filter("Executables", &["exe"])
            .pick_file()
            .context("no executable selected")?,
    };
    validate_executable(&path)?;
    let mut game = Game::local(
        args.name.unwrap_or_else(|| infer_name(&path)),
        path.canonicalize().unwrap_or(path),
    );
    game.launch_args = args.args;
    game.env_vars = parse_env_pairs(&args.env)?;
    game.profile.priority = args.priority.map(Into::into).unwrap_or(default_priority);
    game.profile.affinity_mask = args.affinity.as_deref().map(parse_affinity_mask).transpose()?;
    game.profile.kill_after_launch = if args.kill_after_launch.is_empty() {
        default_kill_list.to_vec()
    } else {
        args.kill_after_launch
    };
    game.updated_at = Utc::now();
    db.upsert_game(&game)?;
    println!("Added {} [{}]", game.name, game.id);
    Ok(())
}

fn scan_games(
    db: &Database,
    args: ScanArgs,
    default_kill_list: &[String],
    default_priority: PriorityClass,
) -> Result<()> {
    let walker = if args.flat {
        WalkDir::new(&args.dir).max_depth(1)
    } else {
        WalkDir::new(&args.dir)
    };
    let mut count = 0_u64;
    for entry in walker.into_iter().filter_map(Result::ok) {
        let path = entry.into_path();
        if !is_candidate_exe(&path) {
            continue;
        }
        let mut game = Game::local(infer_name(&path), path.canonicalize().unwrap_or(path.clone()));
        game.profile.priority = default_priority.clone();
        game.profile.kill_after_launch = default_kill_list.to_vec();
        db.upsert_game(&game)?;
        count += 1;
    }
    println!("Imported {count} local games");
    Ok(())
}

fn list_games(db: &Database, args: ListArgs) -> Result<()> {
    let games = db.list_games()?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&games)?);
        return Ok(());
    }
    for game in games {
        println!(
            "{:<32} {:<8} plays={:<4} {}",
            game.name,
            game.source,
            game.play_count,
            display_path(&game.executable)
        );
    }
    Ok(())
}

fn launch_game(db: &Database, config: &crate::config::AppConfig, args: LaunchArgs) -> Result<()> {
    let games = db.list_games()?;
    let game = resolve_game(&games, &args.query, args.exact)?;
    let backend = backend_for_source(&game.source);
    let target = backend
        .launch(&game, config)?
        .ok_or_else(|| anyhow!("backend could not resolve a launch target"))?;
    let started = Instant::now();
    let child = launcher::launch(&target, &game.profile)?;
    db.mark_launched(&game.id)?;
    println!(
        "Launched {} (pid={}, {} ms)",
        game.name,
        child.id(),
        started.elapsed().as_millis()
    );
    Ok(())
}

fn import_games(db: &Database, config: &crate::config::AppConfig, args: ImportArgs) -> Result<()> {
    let imported = match args.source {
        ImportSource::All => {
            let mut games = import_from_all(config)?;
            for game in &mut games {
                apply_recommended_profile(game, TuneModeArg::Safe);
            }
            for game in &games {
                db.upsert_game(game)?;
            }
            games
        }
        ImportSource::Steam => import_with_backend(db, config, SteamBackend)?,
        ImportSource::Epic => import_with_backend(db, config, LegendaryBackend)?,
        ImportSource::Gog => import_with_backend(db, config, GogBackend)?,
        ImportSource::Amazon => import_with_backend(db, config, AmazonBackend)?,
    };
    println!("Imported {} games", imported.len());
    Ok(())
}

fn configure_game(db: &Database, args: ConfigArgs) -> Result<()> {
    let games = db.list_games()?;
    let mut game = resolve_game(&games, &args.query, false)?;
    let no_mutation = !args.show
        && args.priority.is_none()
        && args.affinity.is_none()
        && args.kill_after_launch.is_empty()
        && args.env.is_empty()
        && !args.clear_env
        && !args.clear_kill_rules;
    if args.show || no_mutation {
        println!("{}", serde_json::to_string_pretty(&game)?);
        return Ok(());
    }
    if let Some(priority) = args.priority {
        game.profile.priority = priority.into();
    }
    if let Some(affinity) = args.affinity {
        game.profile.affinity_mask = Some(parse_affinity_mask(&affinity)?);
    }
    if args.clear_kill_rules {
        game.profile.kill_after_launch.clear();
    }
    if !args.kill_after_launch.is_empty() {
        game.profile.kill_after_launch = args.kill_after_launch;
    }
    if args.clear_env {
        game.profile.env_overrides.clear();
    }
    if !args.env.is_empty() {
        for (key, value) in parse_env_pairs(&args.env)? {
            game.profile.env_overrides.insert(key, value);
        }
    }
    game.updated_at = Utc::now();
    db.update_game(&game)?;
    println!("{}", serde_json::to_string_pretty(&game)?);
    Ok(())
}

fn export_game(
    db: &Database,
    config: &crate::config::AppConfig,
    export_dir: &Path,
    args: ExportArgs,
) -> Result<()> {
    let games = db.list_games()?;
    let game = resolve_game(&games, &args.query, false)?;
    let backend = backend_for_source(&game.source);
    let target = backend
        .resolve_launch_target(&game, config)?
        .ok_or_else(|| anyhow!("backend could not resolve a launch target"))?;
    let output = args
        .output
        .unwrap_or_else(|| export_dir.join(format!("{}.bat", sanitize_filename(&game.name))));
    launcher::build_export_script(&target, &game.profile, &output)?;
    println!("Exported {}", output.display());
    Ok(())
}

fn import_with_backend<B: Backend>(db: &Database, config: &crate::config::AppConfig, backend: B) -> Result<Vec<Game>> {
    if !backend.detect(config) {
        return Ok(Vec::new());
    }
    let mut games = backend.import_games(config)?;
    for game in &mut games {
        apply_recommended_profile(game, TuneModeArg::Safe);
    }
    for game in &games {
        db.upsert_game(game)?;
    }
    Ok(games)
}

fn show_status(db: &Database, config: &crate::config::AppConfig) -> Result<()> {
    let games = db.list_games()?;
    let (mut local, mut steam, mut epic, mut gog, mut amazon) = (0_u64, 0_u64, 0_u64, 0_u64, 0_u64);
    for game in games {
        match game.source {
            GameSource::Local => local += 1,
            GameSource::Steam => steam += 1,
            GameSource::Epic => epic += 1,
            GameSource::Gog => gog += 1,
            GameSource::Amazon => amazon += 1,
        }
    }
    println!("Library: local={local} steam={steam} epic={epic} gog={gog} amazon={amazon}");
    println!("Backends:");
    println!(
        "  steam    : {}",
        if SteamBackend.detect(config) { "ready" } else { "missing (Steam install not detected)" }
    );
    println!(
        "  epic     : {}",
        if LegendaryBackend.detect(config) { "ready (legendary found)" } else { "missing (legendary not found)" }
    );
    println!(
        "  gog      : {}",
        if GogBackend.detect(config) { "ready (gogdl found)" } else { "missing (gogdl not found)" }
    );
    println!(
        "  amazon   : {}",
        if AmazonBackend.detect(config) { "ready (amazon helper found)" } else { "missing (helper not found)" }
    );
    Ok(())
}

fn tune_game(db: &Database, args: TuneArgs) -> Result<()> {
    let games = db.list_games()?;
    let mut game = resolve_game(&games, &args.query, false)?;
    apply_recommended_profile(&mut game, args.mode);
    game.updated_at = Utc::now();
    db.update_game(&game)?;
    println!(
        "Tuned {} -> priority={:?}, kill_rules={}",
        game.name,
        game.profile.priority,
        game.profile.kill_after_launch.join(",")
    );
    Ok(())
}

fn apply_recommended_profile(game: &mut Game, mode: TuneModeArg) {
    if !game.profile.kill_after_launch.is_empty() {
        return;
    }
    let (priority, kill_list): (PriorityClass, Vec<String>) = match (game.source.clone(), mode) {
        (GameSource::Local, TuneModeArg::Safe) => (PriorityClass::AboveNormal, vec![]),
        (GameSource::Local, TuneModeArg::Aggressive) => (
            PriorityClass::High,
            vec![
                "steam.exe".into(),
                "epicgameslauncher.exe".into(),
                "riotclientservices.exe".into(),
            ],
        ),
        (GameSource::Steam, TuneModeArg::Safe) => (
            PriorityClass::AboveNormal,
            vec!["steamwebhelper.exe".into(), "steamservice.exe".into()],
        ),
        (GameSource::Steam, TuneModeArg::Aggressive) => (
            PriorityClass::High,
            vec!["steamwebhelper.exe".into(), "steam.exe".into()],
        ),
        (GameSource::Epic, TuneModeArg::Safe) => (
            PriorityClass::AboveNormal,
            vec!["epicwebhelper.exe".into(), "epicgameslauncher.exe".into()],
        ),
        (GameSource::Epic, TuneModeArg::Aggressive) => (
            PriorityClass::High,
            vec!["epicwebhelper.exe".into(), "epicgameslauncher.exe".into()],
        ),
        (GameSource::Gog, TuneModeArg::Safe) => (
            PriorityClass::AboveNormal,
            vec!["galaxyclient.exe".into(), "galaxyclient helper.exe".into()],
        ),
        (GameSource::Gog, TuneModeArg::Aggressive) => (
            PriorityClass::High,
            vec!["galaxyclient.exe".into(), "galaxyclient helper.exe".into()],
        ),
        (GameSource::Amazon, TuneModeArg::Safe) => (
            PriorityClass::AboveNormal,
            vec!["amazon games ui.exe".into()],
        ),
        (GameSource::Amazon, TuneModeArg::Aggressive) => (
            PriorityClass::High,
            vec!["amazon games ui.exe".into(), "amazon games.exe".into()],
        ),
    };
    game.profile.priority = priority;
    game.profile.kill_after_launch = kill_list;
}

fn validate_executable(path: &Path) -> Result<()> {
    if path.extension().and_then(|value| value.to_str()).map(|ext| ext.eq_ignore_ascii_case("exe")) != Some(true) {
        anyhow::bail!("expected a .exe path");
    }
    if !path.exists() {
        anyhow::bail!("path does not exist: {}", path.display());
    }
    Ok(())
}

fn infer_name(path: &Path) -> String {
    path.file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("Unknown Game")
        .replace(['_', '-'], " ")
}

fn parse_env_pairs(values: &[String]) -> Result<HashMap<String, String>> {
    let mut env = HashMap::new();
    for item in values {
        let Some((key, value)) = item.split_once('=') else {
            anyhow::bail!("invalid env pair '{item}', expected KEY=VALUE");
        };
        env.insert(key.trim().to_string(), value.trim().to_string());
    }
    Ok(env)
}

fn resolve_game(games: &[Game], query: &str, exact: bool) -> Result<Game> {
    if exact {
        return games
            .iter()
            .find(|game| game.name.eq_ignore_ascii_case(query))
            .cloned()
            .ok_or_else(|| anyhow!("no game named '{query}'"));
    }
    if let Some(game) = games.iter().find(|game| game.name.eq_ignore_ascii_case(query)) {
        return Ok(game.clone());
    }
    let matcher = SkimMatcherV2::default().ignore_case();
    let mut matches = games
        .iter()
        .filter_map(|game| matcher.fuzzy_match(&game.name, query).map(|score| (score, game)))
        .collect::<Vec<_>>();
    matches.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.name.cmp(&b.1.name)));
    match matches.as_slice() {
        [] => Err(anyhow!("no game matched '{query}'")),
        [(_, game)] => Ok((*game).clone()),
        [top, second, ..] if top.0 > second.0 + 15 => Ok(top.1.clone()),
        _ => {
            let options = matches
                .iter()
                .take(5)
                .enumerate()
                .map(|(index, (_, game))| format!("{}. {} [{}]", index + 1, game.name, game.source))
                .collect::<Vec<_>>()
                .join("\n");
            Err(anyhow!("query '{query}' is ambiguous:\n{options}"))
        }
    }
}

fn is_candidate_exe(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    if path.extension().and_then(|value| value.to_str()).map(|ext| ext.eq_ignore_ascii_case("exe")) != Some(true) {
        return false;
    }
    let file_name = path.file_name().and_then(|value| value.to_str()).unwrap_or_default().to_ascii_lowercase();
    let skip_patterns = ["unins", "uninstall", "setup", "installer", "crashreporter", "redist"];
    !skip_patterns.iter().any(|pattern| file_name.contains(pattern))
}

fn sanitize_filename(value: &str) -> String {
    value.chars()
        .map(|ch| match ch {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            _ => ch,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{apply_recommended_profile, parse_env_pairs, resolve_game, shell_argv};
    use crate::cli::TuneModeArg;
    use crate::models::{Game, GameSource};

    #[test]
    fn parse_env_pairs_requires_equals() {
        let error = parse_env_pairs(&["FOO".into()]).expect_err("should fail");
        assert!(error.to_string().contains("KEY=VALUE"));
    }

    #[test]
    fn exact_match_wins() {
        let games = vec![
            Game::local("Cyberpunk 2077".into(), PathBuf::from(r"C:\Cyberpunk\game.exe")),
            Game::local("Cyber Hook".into(), PathBuf::from(r"C:\CyberHook\game.exe")),
        ];
        let game = resolve_game(&games, "Cyberpunk 2077", false).expect("game");
        assert_eq!(game.name, "Cyberpunk 2077");
    }

    #[test]
    fn ambiguous_match_returns_error() {
        let games = vec![
            Game::local("Need for Speed".into(), PathBuf::from(r"C:\NFS\game.exe")),
            Game::local("Need for Speed Heat".into(), PathBuf::from(r"C:\NFSH\game.exe")),
        ];
        let error = resolve_game(&games, "need", false).expect_err("ambiguous");
        assert!(error.to_string().contains("ambiguous"));
    }

    #[test]
    fn shell_argv_handles_optional_prefix_and_quotes() {
        let argv = shell_argv(r#"bolt launch "Need for Speed""#).expect("argv");
        assert_eq!(argv, vec!["bolt", "launch", "Need for Speed"]);
    }

    #[test]
    fn tune_profile_sets_epic_kill_rules() {
        let mut game = Game::local("Fortnite".into(), PathBuf::from(r"C:\Games\FN\fn.exe"));
        game.source = GameSource::Epic;
        apply_recommended_profile(&mut game, TuneModeArg::Safe);
        assert!(game.profile.kill_after_launch.iter().any(|v| v == "epicgameslauncher.exe"));
    }
}

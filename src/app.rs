use std::collections::HashMap;
use std::path::Path;

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
use crate::cli::{AddArgs, BANNER, Cli, Commands, ConfigArgs, ExportArgs, ImportArgs, ImportSource, LaunchArgs, ListArgs, ScanArgs};
use crate::config::{load as load_config, resolve_paths};
use crate::db::Database;
use crate::launcher;
use crate::models::{Game, PriorityClass, display_path, parse_affinity_mask};

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    if cli.command.is_none() {
        println!("{BANNER}");
    }
    let paths = resolve_paths()?;
    let _root_dir = &paths.root_dir;
    let config = load_config(&paths)?;
    let db = Database::open(&paths.db_file)?;
    match cli.command.unwrap_or(Commands::List(ListArgs { json: false })) {
        Commands::Add(args) => add_game(&db, args, &config.default_kill_list, config.default_priority),
        Commands::Scan(args) => scan_games(&db, args, &config.default_kill_list, config.default_priority),
        Commands::List(args) => list_games(&db, args),
        Commands::Launch(args) => launch_game(&db, &config, args),
        Commands::Import(args) => import_games(&db, &config, args),
        Commands::Config(args) => configure_game(&db, args),
        Commands::Export(args) => export_game(&db, &config, &paths.export_dir, args),
    }
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
    launcher::launch(&target, &game.profile)?;
    db.mark_launched(&game.id)?;
    println!("Launched {}", game.name);
    Ok(())
}

fn import_games(db: &Database, config: &crate::config::AppConfig, args: ImportArgs) -> Result<()> {
    let imported = match args.source {
        ImportSource::All => {
            let games = import_from_all(config)?;
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
    let games = backend.import_games(config)?;
    for game in &games {
        db.upsert_game(game)?;
    }
    Ok(games)
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

    use super::{parse_env_pairs, resolve_game};
    use crate::models::Game;

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
}

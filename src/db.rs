use std::path::Path;

use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};

use crate::models::{Game, LaunchProfile, LauncherMetadata};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS games (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                source TEXT NOT NULL,
                executable TEXT NOT NULL,
                working_dir TEXT NOT NULL,
                launch_args TEXT NOT NULL,
                env_json TEXT NOT NULL,
                launcher_metadata_json TEXT NOT NULL,
                profile_json TEXT NOT NULL,
                last_played TEXT NULL,
                play_count INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_games_name ON games(name);
            CREATE INDEX IF NOT EXISTS idx_games_source ON games(source);
            "#,
        )?;
        Ok(())
    }

    pub fn upsert_game(&self, game: &Game) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO games (
                id, name, source, executable, working_dir, launch_args, env_json,
                launcher_metadata_json, profile_json, last_played, play_count, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            ON CONFLICT(id) DO UPDATE SET
                name=excluded.name,
                source=excluded.source,
                executable=excluded.executable,
                working_dir=excluded.working_dir,
                launch_args=excluded.launch_args,
                env_json=excluded.env_json,
                launcher_metadata_json=excluded.launcher_metadata_json,
                profile_json=excluded.profile_json,
                updated_at=excluded.updated_at
            "#,
            params![
                game.id,
                game.name,
                game.source.to_string(),
                game.executable.to_string_lossy(),
                game.working_dir.to_string_lossy(),
                serde_json::to_string(&game.launch_args)?,
                serde_json::to_string(&game.env_vars)?,
                serde_json::to_string(&game.launcher_metadata)?,
                serde_json::to_string(&game.profile)?,
                game.last_played.map(|value| value.to_rfc3339()),
                game.play_count as i64,
                game.created_at.to_rfc3339(),
                game.updated_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn update_game(&self, game: &Game) -> Result<()> {
        self.upsert_game(game)
    }

    pub fn list_games(&self) -> Result<Vec<Game>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, name, source, executable, working_dir, launch_args, env_json,
                   launcher_metadata_json, profile_json, last_played, play_count,
                   created_at, updated_at
            FROM games
            ORDER BY LOWER(name), source
            "#,
        )?;
        let rows = stmt.query_map([], row_to_game)?;
        let mut games = Vec::new();
        for row in rows {
            games.push(row?);
        }
        Ok(games)
    }

    pub fn mark_launched(&self, game_id: &str) -> Result<()> {
        self.conn.execute(
            r#"
            UPDATE games
            SET play_count = play_count + 1,
                last_played = ?2,
                updated_at = ?2
            WHERE id = ?1
            "#,
            params![game_id, Utc::now().to_rfc3339()],
        )?;
        Ok(())
    }
}

fn row_to_game(row: &rusqlite::Row<'_>) -> rusqlite::Result<Game> {
    let source: String = row.get(2)?;
    let last_played: Option<String> = row.get(9)?;
    let created_at: String = row.get(11)?;
    let updated_at: String = row.get(12)?;
    Ok(Game {
        id: row.get(0)?,
        name: row.get(1)?,
        source: source
            .parse::<crate::models::GameSource>()
            .map_err(|error| to_sql_error(message_error(error.to_string())))?,
        executable: row.get::<_, String>(3)?.into(),
        working_dir: row.get::<_, String>(4)?.into(),
        launch_args: serde_json::from_str(&row.get::<_, String>(5)?).map_err(to_sql_error)?,
        env_vars: serde_json::from_str(&row.get::<_, String>(6)?).map_err(to_sql_error)?,
        launcher_metadata: serde_json::from_str::<LauncherMetadata>(&row.get::<_, String>(7)?)
            .map_err(to_sql_error)?,
        profile: serde_json::from_str::<LaunchProfile>(&row.get::<_, String>(8)?).map_err(to_sql_error)?,
        last_played: last_played.map(|value| parse_dt(&value)).transpose().map_err(to_sql_error)?,
        play_count: row.get::<_, i64>(10)? as u64,
        created_at: parse_dt(&created_at).map_err(to_sql_error)?,
        updated_at: parse_dt(&updated_at).map_err(to_sql_error)?,
    })
}

fn parse_dt(value: &str) -> std::result::Result<DateTime<Utc>, chrono::ParseError> {
    Ok(DateTime::parse_from_rfc3339(value)?.with_timezone(&Utc))
}

fn to_sql_error(error: impl std::error::Error + Send + Sync + 'static) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error))
}

fn message_error(message: String) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, message)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::Database;
    use crate::models::Game;

    #[test]
    fn upsert_preserves_single_row_for_same_game() {
        let db = Database::open_in_memory().expect("db");
        let mut game = Game::local("Halo".into(), PathBuf::from(r"C:\Games\Halo\halo.exe"));
        db.upsert_game(&game).expect("insert");
        game.name = "Halo CE".into();
        db.upsert_game(&game).expect("update");
        let games = db.list_games().expect("list");
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].name, "Halo CE");
    }
}

//! Turso/libSQL database connection and schema.
//!
//! Uses a single database connection (no pool) to prevent hrana
//! protocol race conditions that occur with concurrent connections.

use libsql::{Connection, Database};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Shared database handle with a single pre-created connection.
/// Clone is cheap (Arc).
#[derive(Clone)]
pub struct DbPool {
    pub conn: Arc<Connection>,
}

impl DbPool {
    /// Connect to database from environment variables.
    ///
    /// Reads `DATABASE_URL` and (for remote) `DATABASE_AUTH_TOKEN`.
    /// Falls back to `file:local.db` if `DATABASE_URL` is not set.
    pub async fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "file:local.db".into());

        let db = if url.starts_with("libsql://") || url.starts_with("https://") {
            let auth_token = std::env::var("DATABASE_AUTH_TOKEN")
                .map_err(|_| "DATABASE_AUTH_TOKEN required for remote Turso connection".to_string())?;

            Database::open_remote(url.clone(), auth_token)?
        } else {
            Database::open(url.clone())?
        };

        // Create a single connection at startup and reuse it for all requests.
        // This avoids hrana protocol race conditions from concurrent connections.
        let conn = db.connect()?;
        tracing::info!(
            "Connected to database: {}",
            if url.starts_with("libsql://") || url.starts_with("https://") {
                "Turso remote (single connection)"
            } else {
                "local SQLite"
            }
        );
        Ok(DbPool {
            conn: Arc::new(conn),
        })
    }

    /// Get the shared connection for executing queries.
    pub async fn conn(&self) -> &Connection {
        &self.conn
    }
}

// ── Schema Migration ─────────────────────────────────────────────────────────

const SCHEMA_SQL: &str = "
CREATE TABLE IF NOT EXISTS users (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    username        TEXT NOT NULL UNIQUE,
    password        TEXT NOT NULL,
    games_played    INTEGER NOT NULL DEFAULT 0,
    games_won       INTEGER NOT NULL DEFAULT 0,
    total_sets_won  INTEGER NOT NULL DEFAULT 0,
    most_sets_won   INTEGER NOT NULL DEFAULT 0,
    elo             INTEGER NOT NULL DEFAULT 500
);

CREATE TABLE IF NOT EXISTS matches (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id         INTEGER NOT NULL REFERENCES users(id),
    date            INTEGER NOT NULL,
    bot_difficulty  TEXT NOT NULL,

    -- Betting info
    trump_suit      TEXT NOT NULL,
    bet_size        INTEGER NOT NULL,
    bet_winner      INTEGER NOT NULL,

    -- Match result
    partner         INTEGER,
    won_match       INTEGER,

    -- Sets won per player
    player1_sets    INTEGER NOT NULL DEFAULT 0,
    player2_sets    INTEGER NOT NULL DEFAULT 0,
    player3_sets    INTEGER NOT NULL DEFAULT 0,
    player4_sets    INTEGER NOT NULL DEFAULT 0,

    -- Serialized hands (JSON array of cards)
    player1_hand    TEXT NOT NULL DEFAULT '[]',
    player2_hand    TEXT NOT NULL DEFAULT '[]',
    player3_hand    TEXT NOT NULL DEFAULT '[]',
    player4_hand    TEXT NOT NULL DEFAULT '[]',

    -- Completed sets data (JSON)
    sets_data       TEXT,
    players         TEXT,
    players_int     INTEGER DEFAULT 0,
    room_id         TEXT,
    elo_change      INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS sessions (
    id          TEXT PRIMARY KEY,
    user_id     INTEGER NOT NULL REFERENCES users(id),
    expires_at  INTEGER NOT NULL
);
";

/// Run the initial schema migration.
pub async fn run_migrations(pool: &DbPool) -> Result<(), Box<dyn std::error::Error>> {
    let conn = pool.conn().await;
    conn.execute_batch(SCHEMA_SQL).await?;

    // Safe migration: add columns if they don't exist yet
    let _ = conn.execute_batch("ALTER TABLE matches ADD COLUMN sets_data TEXT;").await;
    let _ = conn.execute_batch("ALTER TABLE matches ADD COLUMN players TEXT;").await;
    let _ = conn.execute_batch("ALTER TABLE matches ADD COLUMN room_id TEXT;").await;
    let _ = conn.execute_batch("ALTER TABLE matches ADD COLUMN players_int INTEGER DEFAULT 0;").await;
    let _ = conn.execute_batch("ALTER TABLE matches ADD COLUMN bet_winner_user_id INTEGER DEFAULT 0;").await;
    let _ = conn.execute_batch("ALTER TABLE matches ADD COLUMN partner_user_id INTEGER DEFAULT 0;").await;
    let _ = conn.execute_batch("ALTER TABLE matches ADD COLUMN winning_team INTEGER DEFAULT 1;").await;
    let _ = conn.execute_batch("ALTER TABLE users ADD COLUMN games_played INTEGER DEFAULT 0;").await;
    let _ = conn.execute_batch("ALTER TABLE users ADD COLUMN games_won INTEGER DEFAULT 0;").await;
    let _ = conn.execute_batch("ALTER TABLE users ADD COLUMN total_sets_won INTEGER DEFAULT 0;").await;
    let _ = conn.execute_batch("ALTER TABLE users ADD COLUMN most_sets_won INTEGER DEFAULT 0;").await;
    let _ = conn.execute_batch("ALTER TABLE users ADD COLUMN elo INTEGER DEFAULT 500;").await;
    let _ = conn.execute_batch("ALTER TABLE matches ADD COLUMN elo_change INTEGER DEFAULT 0;").await;

    // Seed bot users (standard bots: Alpha, Beta, Gamma)
    let _ = conn.execute(
        "INSERT OR IGNORE INTO users (id, username, password, games_played, games_won, total_sets_won, most_sets_won, elo) VALUES (?1, ?2, '', 0, 0, 0, 0, 500)",
        libsql::params![1i64, "Bot-Alpha"],
    ).await;
    let _ = conn.execute(
        "INSERT OR IGNORE INTO users (id, username, password, games_played, games_won, total_sets_won, most_sets_won, elo) VALUES (?1, ?2, '', 0, 0, 0, 0, 500)",
        libsql::params![2i64, "Bot-Beta"],
    ).await;
    let _ = conn.execute(
        "INSERT OR IGNORE INTO users (id, username, password, games_played, games_won, total_sets_won, most_sets_won, elo) VALUES (?1, ?2, '', 0, 0, 0, 0, 500)",
        libsql::params![3i64, "Bot-Gamma"],
    ).await;

    tracing::info!("Database schema up to date");
    Ok(())
}

// ── Row Types ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct UserRow {
    pub id: i64,
    pub username: String,
    pub password: String,
    pub games_played: i64,
    pub games_won: i64,
    pub total_sets_won: i64,
    pub most_sets_won: i64,
    pub elo: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
#[serde(rename_all = "camelCase")]
pub struct MatchRow {
    pub id: i64,
    pub user_id: i64,
    pub date: i64,
    pub bot_difficulty: String,
    pub trump_suit: String,
    pub bet_size: i64,
    pub bet_winner: i64,
    pub partner: Option<i64>,
    pub won_match: Option<i64>,
    pub bet_winner_user_id: i64,
    pub partner_user_id: i64,
    pub winning_team: i64,
    pub player1_sets: i64,
    pub player2_sets: i64,
    pub player3_sets: i64,
    pub player4_sets: i64,
    pub player1_hand: String,
    pub player2_hand: String,
    pub player3_hand: String,
    pub player4_hand: String,
    pub sets_data: Option<String>,
    pub players: Option<String>,
    pub players_int: i64,
    pub room_id: Option<String>,
    pub elo_change: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct SessionRow {
    pub id: String,
    pub user_id: i64,
    pub expires_at: i64,
}

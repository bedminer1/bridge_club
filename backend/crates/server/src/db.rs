//! Turso/libSQL database connection and schema.
//!
//! Mirrors the frontend's Drizzle schema (users, matches, sessions).
//! Uses the `libsql` crate for async SQLite-over-HTTP or local SQLite.

use libsql::Database;
use std::sync::Arc;

/// Shared database handle.
/// Wraps Arc<Database> because libsql::Database is not Clone.
#[derive(Clone)]
pub struct DbPool {
    pub db: Arc<Database>,
}

impl DbPool {
    /// Connect to Turso from environment variables.
    ///
    /// Reads `DATABASE_URL` and (for remote) `DATABASE_AUTH_TOKEN`.
    /// Falls back to `file:local.db` if `DATABASE_URL` is not set.
    pub async fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "file:local.db".into());

        let db = if url.starts_with("libsql://") || url.starts_with("https://") {
            // Remote Turso connection
            let auth_token = std::env::var("DATABASE_AUTH_TOKEN")
                .map_err(|_| "DATABASE_AUTH_TOKEN required for remote Turso connection".to_string())?;

            #[allow(deprecated)]
            Database::open_remote(url.clone(), auth_token)?
        } else {
            // Local SQLite file
            #[allow(deprecated)]
            Database::open(url.clone())?
        };

        tracing::info!(
            "Connected to database: {}",
            if url.starts_with("libsql://") { "Turso remote" } else { "local SQLite" }
        );
        Ok(DbPool { db: Arc::new(db) })
    }

    /// Get a connection from the pool.
    pub async fn conn(&self) -> Result<libsql::Connection, libsql::Error> {
        self.db.connect()
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
    most_sets_won   INTEGER NOT NULL DEFAULT 0
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
    room_id         TEXT
);

CREATE TABLE IF NOT EXISTS sessions (
    id          TEXT PRIMARY KEY,
    user_id     INTEGER NOT NULL REFERENCES users(id),
    expires_at  INTEGER NOT NULL
);
";

/// Run the initial schema migration.
pub async fn run_migrations(pool: &DbPool) -> Result<(), Box<dyn std::error::Error>> {
    let conn = pool.conn().await?;
    conn.execute_batch(SCHEMA_SQL).await?;

    // Safe migration: add columns if they don't exist yet
    let _ = conn.execute_batch("ALTER TABLE matches ADD COLUMN sets_data TEXT;").await;
    let _ = conn.execute_batch("ALTER TABLE matches ADD COLUMN players TEXT;").await;
    let _ = conn.execute_batch("ALTER TABLE matches ADD COLUMN room_id TEXT;").await;
    let _ = conn.execute_batch("ALTER TABLE matches ADD COLUMN players_int INTEGER DEFAULT 0;").await;
    let _ = conn.execute_batch("ALTER TABLE users ADD COLUMN games_played INTEGER DEFAULT 0;").await;
    let _ = conn.execute_batch("ALTER TABLE users ADD COLUMN games_won INTEGER DEFAULT 0;").await;
    let _ = conn.execute_batch("ALTER TABLE users ADD COLUMN total_sets_won INTEGER DEFAULT 0;").await;
    let _ = conn.execute_batch("ALTER TABLE users ADD COLUMN most_sets_won INTEGER DEFAULT 0;").await;

    tracing::info!("Database schema up to date");
    Ok(())
}

// ── Row Types ────────────────────────────────────────────────────────────────

use serde::{Deserialize, Serialize};

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct SessionRow {
    pub id: String,
    pub user_id: i64,
    pub expires_at: i64,
}

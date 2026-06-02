//! Turso/libSQL database connection and schema.
//!
//! Serializes all database access through a Mutex. Auto-reconnects when the
//! hrana stream is closed by the Turso server.

use libsql::{Builder, Connection, Database};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Shared database handle with auto-reconnecting connection.
#[derive(Clone)]
pub struct DbPool {
    db: Arc<Database>,
    conn: Arc<Mutex<Connection>>,
}

impl DbPool {
    /// Connect to database from environment variables.
    pub async fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "file:local.db".into());

        let db = if url.starts_with("libsql://") || url.starts_with("https://") {
            let auth_token = std::env::var("DATABASE_AUTH_TOKEN")
                .map_err(|_| "DATABASE_AUTH_TOKEN required for remote Turso connection".to_string())?;
            Builder::new_remote(url.clone(), auth_token).build().await?
        } else {
            Builder::new_local(url.clone()).build().await?
        };

        let conn = db.connect()?;
        tracing::info!(
            "Connected to database: {}",
            if url.starts_with("libsql://") || url.starts_with("https://") { "Turso remote" } else { "local SQLite" }
        );
        Ok(DbPool {
            db: Arc::new(db),
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Get a serialized, fresh connection for executing queries.
    /// Creates a new hrana connection each call (serialized via Mutex) so
    /// that stale/closed streams are never reused.
    pub async fn conn(&self) -> tokio::sync::MutexGuard<'_, Connection> {
        // Always create a fresh connection to avoid "stream not found" errors
        if let Ok(new_conn) = self.db.connect() {
            let mut guard = self.conn.lock().await;
            *guard = new_conn;
        }
        self.conn.lock().await
    }

    /// Replace the current connection with a fresh one (for error recovery).
    pub async fn reconnect(&self) {
        if let Ok(new_conn) = self.db.connect() {
            let mut guard = self.conn.lock().await;
            *guard = new_conn;
        }
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
    trump_suit      TEXT NOT NULL,
    bet_size        INTEGER NOT NULL,
    bet_winner      INTEGER NOT NULL,
    partner         INTEGER,
    won_match       INTEGER,
    player1_sets    INTEGER NOT NULL DEFAULT 0,
    player2_sets    INTEGER NOT NULL DEFAULT 0,
    player3_sets    INTEGER NOT NULL DEFAULT 0,
    player4_sets    INTEGER NOT NULL DEFAULT 0,
    player1_hand    TEXT NOT NULL DEFAULT '[]',
    player2_hand    TEXT NOT NULL DEFAULT '[]',
    player3_hand    TEXT NOT NULL DEFAULT '[]',
    player4_hand    TEXT NOT NULL DEFAULT '[]',
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

CREATE TABLE IF NOT EXISTS match_participants (
    match_id    INTEGER NOT NULL REFERENCES matches(id),
    user_id     INTEGER NOT NULL REFERENCES users(id),
    seat_index  INTEGER NOT NULL,
    PRIMARY KEY (match_id, user_id)
);
";

pub async fn run_migrations(pool: &DbPool) -> Result<(), Box<dyn std::error::Error>> {
    let conn = pool.conn().await;
    conn.execute_batch(SCHEMA_SQL).await?;

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

    // Create match_participants join table for indexed user->match lookups
    let _ = conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS match_participants (\n         match_id INTEGER NOT NULL REFERENCES matches(id),\n         user_id INTEGER NOT NULL REFERENCES users(id),\n         seat_index INTEGER NOT NULL,\n         PRIMARY KEY (match_id, user_id)\n         );\n         CREATE INDEX IF NOT EXISTS idx_match_participants_user_id ON match_participants(user_id);\n         "
    ).await;

    tracing::info!("Database schema up to date");
    Ok(())
}

/// Create a test pool backed by a temporary SQLite file.
pub async fn new_temp(path: &str) -> Result<DbPool, Box<dyn std::error::Error>> {
    let db = Builder::new_local(path.to_string()).build().await?;
    let conn = db.connect()?;
    Ok(DbPool {
        db: Arc::new(db),
        conn: Arc::new(Mutex::new(conn)),
    })
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

/// Lightweight match row without hand/sets blobs — used for match list queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
#[serde(rename_all = "camelCase")]
pub struct MatchRowLight {
    pub id: i64,
    pub date: i64,
    pub bot_difficulty: String,
    pub trump_suit: String,
    pub bet_size: i64,
    pub bet_winner_user_id: i64,
    pub partner_user_id: i64,
    pub winning_team: i64,
    pub won_match: Option<i64>,
    pub player1_sets: i64,
    pub player2_sets: i64,
    pub player3_sets: i64,
    pub player4_sets: i64,
    pub players: Option<String>,
    pub elo_change: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct SessionRow {
    pub id: String,
    pub user_id: i64,
    pub expires_at: i64,
}

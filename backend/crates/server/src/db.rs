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
    room_id         TEXT UNIQUE,
    created_at      INTEGER NOT NULL,
    trump_suit      TEXT NOT NULL,
    bet_size        INTEGER NOT NULL,
    bet_winner_idx  INTEGER NOT NULL,
    partner_idx     INTEGER,
    partner_card    TEXT,
    winning_team    INTEGER NOT NULL,
    team1_sets      INTEGER NOT NULL DEFAULT 0,
    team2_sets      INTEGER NOT NULL DEFAULT 0,
    sets_data       TEXT,
    match_type      TEXT NOT NULL DEFAULT 'multi',
    is_hidden       INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS match_participants (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    match_id        INTEGER NOT NULL REFERENCES matches(id) ON DELETE CASCADE,
    user_id         INTEGER NOT NULL REFERENCES users(id),
    seat_index      INTEGER NOT NULL CHECK(seat_index BETWEEN 0 AND 3),
    team            INTEGER NOT NULL CHECK(team IN (1, 2)),
    sets_won        INTEGER NOT NULL DEFAULT 0,
    cards_played    TEXT NOT NULL DEFAULT '[]',
    hand_preview    TEXT,
    elo_change      INTEGER NOT NULL DEFAULT 0,
    UNIQUE(match_id, seat_index)
);

CREATE TABLE IF NOT EXISTS sessions (
    id          TEXT PRIMARY KEY,
    user_id     INTEGER NOT NULL REFERENCES users(id),
    expires_at  INTEGER NOT NULL
);
";

pub async fn run_migrations(pool: &DbPool) -> Result<(), Box<dyn std::error::Error>> {
    let conn = pool.conn().await;
    conn.execute_batch(SCHEMA_SQL).await?;

    // Seed bot users
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

/// A match + its participants, as returned by the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
#[serde(rename_all = "camelCase")]
pub struct MatchResponse {
    pub id: i64,
    pub room_id: Option<String>,
    pub created_at: i64,
    pub trump_suit: String,
    pub bet_size: i64,
    pub bet_winner_idx: i64,
    pub partner_idx: Option<i64>,
    pub partner_card: Option<String>,
    pub winning_team: i64,
    pub team1_sets: i64,
    pub team2_sets: i64,
    pub sets_data: Option<String>,
    pub match_type: String,
    pub is_hidden: bool,
    pub participants: Vec<ParticipantResponse>,
}

/// Per-player data in a match.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
#[serde(rename_all = "camelCase")]
pub struct ParticipantResponse {
    pub id: i64,
    pub user_id: i64,
    pub seat_index: i64,
    pub team: i64,
    pub sets_won: i64,
    pub cards_played: String,
    pub hand_preview: Option<String>,
    pub elo_change: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct SessionRow {
    pub id: String,
    pub user_id: i64,
    pub expires_at: i64,
}

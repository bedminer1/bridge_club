use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use game_core::Table;

// ── Session ────────────────────────────────────────────────────────────────

/// Identifies a single connected player within a game session.
/// A session represents one seat at the table.
#[derive(Debug, Clone)]
pub struct PlayerSession {
    pub player_id: Uuid,
    pub player_name: String,
    pub seat_index: usize, // 0..3
}

// ── GameRoom ───────────────────────────────────────────────────────────────

/// A game room holding one table and its seated players.
#[derive(Debug, Clone)]
pub struct GameRoom {
    pub room_id: Uuid,
    pub table: Table,
    pub sessions: HashMap<Uuid, PlayerSession>, // player_id → session
    pub is_started: bool,
}

impl GameRoom {
    pub fn new() -> Self {
        GameRoom {
            room_id: Uuid::new_v4(),
            table: Table::new(["North", "East", "South", "West"]),
            sessions: HashMap::new(),
            is_started: false,
        }
    }

    /// Add a player to this room. Returns their seat index (0..3) or error
    /// if the room is full.
    pub fn add_player(&mut self, player_name: &str) -> Result<(Uuid, usize), &'static str> {
        // TODO:
        // 1. Check if room is full (sessions.len() < 4)
        // 2. Generate player_id (Uuid::new_v4())
        // 3. Assign seat index (smallest available 0..3)
        // 4. Update player name on the table
        // 5. Insert into sessions
        // 6. Return (player_id, seat_index)
        todo!("GameRoom::add_player")
    }

    /// Remove a player from the room.
    pub fn remove_player(&mut self, player_id: Uuid) {
        // TODO: remove from sessions, reset seat name on table
        todo!("GameRoom::remove_player")
    }

    /// Number of seated players (0..4).
    pub fn player_count(&self) -> usize {
        self.sessions.len()
    }

    /// All players are seated and the game can begin.
    pub fn is_ready(&self) -> bool {
        self.sessions.len() == 4 && !self.is_started
    }
}

// ── Global Game State ──────────────────────────────────────────────────────

/// Shared mutable state for the entire server — a map of room_id → GameRoom.
pub type AppState = Arc<RwLock<HashMap<Uuid, GameRoom>>>;

/// Create a fresh shared state.
pub fn new_app_state() -> AppState {
    Arc::new(RwLock::new(HashMap::new()))
}

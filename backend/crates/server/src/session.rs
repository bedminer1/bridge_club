use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use game_core::Table;

use crate::db::DbPool;

// ── Chat Message ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub id: u64,
    pub player_name: String,
    pub text: String,
    pub timestamp: i64,
}

// ── Session ────────────────────────────────────────────────────────────────

/// Identifies a single connected player within a game session.
/// A session represents one seat at the table.
#[derive(Debug, Clone)]
#[allow(dead_code)]
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
    pub sessions: HashMap<Uuid, PlayerSession>,
    pub is_started: bool,
    pub hidden_mode: bool,
    pub messages: Vec<ChatMessage>,
    next_msg_id: u64,
}

impl GameRoom {
    pub fn new() -> Self {
        GameRoom {
            room_id: Uuid::new_v4(),
            table: Table::new(["North", "East", "South", "West"]),
            sessions: HashMap::new(),
            is_started: false,
            hidden_mode: true,
            messages: Vec::new(),
            next_msg_id: 1,
        }
    }

    /// Add a chat message. Returns the message with assigned ID and timestamp.
    pub fn add_message(&mut self, player_name: &str, text: &str) -> ChatMessage {
        let msg = ChatMessage {
            id: self.next_msg_id,
            player_name: player_name.to_string(),
            text: text.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64,
        };
        self.next_msg_id += 1;
        self.messages.push(msg.clone());
        // Keep only last 200 messages
        if self.messages.len() > 200 {
            self.messages.remove(0);
        }
        msg
    }

    /// Add a player to this room. Returns their seat index (0..3) or error
    /// if the room is full.
    pub fn add_player(&mut self, player_name: &str) -> Result<(Uuid, usize), &'static str> {
        if self.sessions.len() >= 4 {
            return Err("Room is full (max 4 players)");
        }
        if self.is_started {
            return Err("Game already started");
        }

        let player_id = Uuid::new_v4();
        // Find smallest available seat index
        let mut taken = [false; 4];
        for session in self.sessions.values() {
            taken[session.seat_index] = true;
        }
        let seat_index = taken.iter().position(|&t| !t).unwrap();

        // Update the player name on the table
        self.table.players[seat_index].name = player_name.to_string();

        self.sessions.insert(
            player_id,
            PlayerSession {
                player_id,
                player_name: player_name.to_string(),
                seat_index,
            },
        );

        Ok((player_id, seat_index))
    }

    /// Remove a player from the room.
    pub fn remove_player(&mut self, player_id: Uuid) {
        if let Some(session) = self.sessions.remove(&player_id) {
            // Reset the player name on the table
            let default_names = ["North", "East", "South", "West"];
            self.table.players[session.seat_index].name =
                default_names[session.seat_index].to_string();
        }
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

// ── Global Application State ───────────────────────────────────────────────

/// Shared mutable state for the entire server.
#[derive(Clone)]
pub struct AppState {
    /// In-memory game rooms (ephemeral game state).
    pub rooms: Arc<RwLock<HashMap<Uuid, GameRoom>>>,
    /// Persistent database pool for users, matches, sessions.
    #[allow(dead_code)]
    pub db: DbPool,
}

/// Create a fresh shared state with a database pool.
pub async fn new_app_state(db_pool: DbPool) -> AppState {
    AppState {
        rooms: Arc::new(RwLock::new(HashMap::new())),
        db: db_pool,
    }
}

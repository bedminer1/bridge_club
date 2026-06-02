//! WebSocket handler for real-time game communication.
//!
//! Replaces HTTP polling for lobby, game state, and chat with a single
//! persistent WebSocket connection per client. The server pushes state
//! updates via broadcast channels per room.

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::{Query, State},
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::auth;
use crate::bot::BotDifficulty;
use crate::game_session::{self, HumanAction};
use crate::routes::TableStateResponse;
use crate::session::AppState;

// ── WebSocket message types ─────────────────────────────────────────────

/// Messages the client sends to the server.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum ClientMessage {
    #[serde(rename = "auth")]
    Auth { token: String },
    #[serde(rename = "lobby:create")]
    LobbyCreate,
    #[serde(rename = "lobby:join")]
    LobbyJoin { room_id: String },
    #[serde(rename = "lobby:leave")]
    LobbyLeave { _player_id: String },
    #[serde(rename = "lobby:start")]
    LobbyStart {
        hidden_mode: Option<bool>,
        difficulty: Option<String>,
    },
    #[serde(rename = "lobby:toggle_hidden")]
    LobbyToggleHidden { enabled: bool },
    #[serde(rename = "game:action")]
    GameAction {
        action_type: String,
        call: Option<serde_json::Value>,
        card: Option<serde_json::Value>,
    },
    #[serde(rename = "chat:send")]
    ChatSend { player_id: String, text: String },
}

/// Messages the server sends to the client.
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum ServerMessage {
    #[serde(rename = "auth:ok")]
    AuthOk {
        #[serde(rename = "userId")]
        user_id: i64,
        username: String,
    },
    #[serde(rename = "lobby:created")]
    LobbyCreated {
        #[serde(rename = "roomId")]
        room_id: Uuid,
        #[serde(rename = "playerId")]
        player_id: Uuid,
        #[serde(rename = "seatIndex")]
        seat_index: usize,
    },
    #[serde(rename = "lobby:joined")]
    LobbyJoined {
        #[serde(rename = "roomId")]
        room_id: Uuid,
        #[serde(rename = "playerId")]
        player_id: Uuid,
        #[serde(rename = "seatIndex")]
        seat_index: usize,
    },
    #[serde(rename = "lobby:update")]
    LobbyUpdate {
        players: Vec<LobbyPlayerInfo>,
        #[serde(rename = "hiddenMode")]
        hidden_mode: bool,
    },
    #[serde(rename = "lobby:started")]
    LobbyStarted {
        #[serde(rename = "roomId")]
        room_id: Uuid,
    },
    #[serde(rename = "game:state")]
    GameState {
        state: TableStateResponse,
        #[serde(rename = "roomId")]
        room_id: Uuid,
    },
    #[serde(rename = "chat:message")]
    ChatMessage {
        id: u64,
        #[serde(rename = "playerName")]
        player_name: String,
        text: String,
        timestamp: i64,
    },
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LobbyPlayerInfo {
    pub name: String,
    pub seat_index: usize,
    pub is_bot: bool,
}

// ── Per-connection state ───────────────────────────────────────────────

struct ConnectionState {
    /// Authenticated user ID (None before auth).
    user_id: Option<i64>,
    username: Option<String>,
    /// Current room the connection is subscribed to.
    current_room: Option<Uuid>,
    /// Player ID in the current room (for lobby operations).
    player_id: Option<Uuid>,
}

// ── WebSocket upgrade handler ─────────────────────────────────────────

#[derive(Deserialize)]
pub struct WsQuery {
    pub token: Option<String>,
}

/// GET /ws — upgrade to WebSocket.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(_query): Query<WsQuery>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handle a single WebSocket connection.
async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    let mut conn = ConnectionState {
        user_id: None,
        username: None,
        current_room: None,
        player_id: None,
    };

    // Broadcast receiver handle — dropped when leaving a room.
    let mut broadcast_rx: Option<broadcast::Receiver<String>> = None;
    let state_clone = state.clone();

    // Spawn a task to read from broadcast and forward to the WebSocket.
    let _broadcast_task = tokio::spawn(async move {
        // This task is managed via broadcast_rx in the main loop below.
        // We use a separate approach: the main loop does the sending.
        let _ = state_clone;
    });

    // Main message loop
    loop {
        tokio::select! {
            // Read from WebSocket
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        tracing::info!(raw_msg = %text, "WS received text message");
                        let response = handle_client_message(&text, &mut conn, &state).await;
                        match response {
                            Ok(resp) => {
                                if let Some(json) = resp {
                                    if sender.send(Message::Text(json.into())).await.is_err() {
                                        break;
                                    }
                                }
                            }
                            Err(err_json) => {
                                let _ = sender.send(Message::Text(err_json.into())).await;
                            }
                        }
                        // Subscribe to broadcast channel when entering a room
                        if broadcast_rx.is_none() {
                            if let Some(room_id) = conn.current_room {
                                broadcast_rx = state
                                    .room_broadcast
                                    .read()
                                    .await
                                    .get(&room_id)
                                    .map(|tx| tx.subscribe());
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
            // Read from broadcast channel (if subscribed to a room)
            else => {
                // If we have a broadcast receiver, check it
                if let Some(ref mut rx) = broadcast_rx {
                    match rx.try_recv() {
                        Ok(msg) => {
                            if sender.send(Message::Text(msg.into())).await.is_err() {
                                break;
                            }
                        }
                        Err(broadcast::error::TryRecvError::Closed) => break,
                        Err(broadcast::error::TryRecvError::Lagged(n)) => {
                            tracing::warn!("Broadcast lagged by {} messages", n);
                        }
                        Err(broadcast::error::TryRecvError::Empty) => {}
                    }
                }
                // Yield to avoid busy-loop
                tokio::task::yield_now().await;
            }
        }
    }

    // Cleanup: remove from broadcast channel
    if let Some(room_id) = conn.current_room {
        let rooms = state.rooms.write().await;
        if let Some(_room) = rooms.get(&room_id) {
            if let Some(pid) = conn.player_id {
                // Don't remove from room on WS disconnect — other player or reconnect
                // Just note the disconnect
                tracing::info!(
                    "Player disconnected from room {} (player {})",
                    room_id, pid
                );
            }
        }
    }
}

/// Handle a single client message, return JSON response or error.
async fn handle_client_message(
    text: &str,
    conn: &mut ConnectionState,
    state: &AppState,
) -> Result<Option<String>, String> {
    let msg: ClientMessage = serde_json::from_str(text).map_err(|e| {
        format!("{{\"type\":\"error\",\"error\":\"Invalid message: {}\"}}", e)
    })?;

    match msg {
        ClientMessage::Auth { token } => {
            handle_auth(token, conn, state).await
        }
        ClientMessage::LobbyCreate => {
            require_auth(conn)?;
            handle_lobby_create(conn, state).await
        }
        ClientMessage::LobbyJoin { room_id } => {
            require_auth(conn)?;
            handle_lobby_join(room_id, conn, state).await
        }
        ClientMessage::LobbyLeave { .. } => {
            handle_lobby_leave(conn, state).await
        }
        ClientMessage::LobbyStart { hidden_mode, difficulty } => {
            let difficulty = difficulty
                .as_deref()
                .and_then(|d| match d.to_lowercase().as_str() {
                    "easy" => Some(BotDifficulty::Easy),
                    "medium" => Some(BotDifficulty::Medium),
                    _ => None,
                })
                .unwrap_or(BotDifficulty::Easy);
            handle_lobby_start(hidden_mode.unwrap_or(true), difficulty, conn, state).await
        }
        ClientMessage::LobbyToggleHidden { enabled } => {
            handle_lobby_toggle_hidden(enabled, conn, state).await
        }
        ClientMessage::GameAction {
            action_type,
            call,
            card,
        } => {
            require_auth(conn)?;
            handle_game_action(action_type, call, card, conn, state).await
        }
        ClientMessage::ChatSend { player_id, text } => {
            handle_chat_send(player_id, text, conn, state).await
        }
    }
}

fn require_auth(conn: &ConnectionState) -> Result<(), String> {
    if conn.user_id.is_none() {
        Err("{\"type\":\"error\",\"error\":\"Not authenticated\"}".to_string())
    } else {
        Ok(())
    }
}

// ── Auth ───────────────────────────────────────────────────────────────

async fn handle_auth(
    token: String,
    conn: &mut ConnectionState,
    state: &AppState,
) -> Result<Option<String>, String> {
    match auth::validate_session(&state.db, &token).await {
        Ok(Some((user, _session))) => {
            conn.user_id = Some(user.id);
            conn.username = Some(user.username.clone());
            Ok(Some(
                serde_json::to_string(&ServerMessage::AuthOk {
                    user_id: user.id,
                    username: user.username,
                })
                .unwrap(),
            ))
        }
        Ok(None) => Err(
            "{\"type\":\"auth:error\",\"error\":\"Invalid or expired session\"}".to_string(),
        ),
        Err(e) => {
            tracing::error!("Auth error: {}", e);
            Err("{\"type\":\"auth:error\",\"error\":\"Internal error\"}".to_string())
        }
    }
}

// ── Lobby ──────────────────────────────────────────────────────────────

async fn handle_lobby_create(
    conn: &mut ConnectionState,
    state: &AppState,
) -> Result<Option<String>, String> {
    let username = conn.username.as_deref().unwrap_or("Player");
    let mut room = crate::session::GameRoom::new();
    let (player_id, seat_index) = room.add_player(username).unwrap();
    let room_id = room.room_id;

    // Create broadcast channel for this room
    let (tx, _) = broadcast::channel(256);
    state.room_broadcast.write().await.insert(room_id, tx);

    let mut rooms = state.rooms.write().await;
    rooms.insert(room_id, room);

    conn.current_room = Some(room_id);
    conn.player_id = Some(player_id);

    tracing::info!("WS lobby created: room={}, player={}", room_id, player_id);

    Ok(Some(
        serde_json::to_string(&ServerMessage::LobbyCreated {
            room_id,
            player_id,
            seat_index,
        })
        .unwrap(),
    ))
}

async fn handle_lobby_join(
    room_id_str: String,
    conn: &mut ConnectionState,
    state: &AppState,
) -> Result<Option<String>, String> {
    let room_id = Uuid::parse_str(&room_id_str).map_err(|_| {
        "{\"type\":\"error\",\"error\":\"Invalid room ID\"}".to_string()
    })?;

    let username = conn.username.as_deref().unwrap_or("Player");

    let mut rooms = state.rooms.write().await;
    let room = rooms.get_mut(&room_id).ok_or_else(|| {
        "{\"type\":\"error\",\"error\":\"Room not found\"}".to_string()
    })?;

    let (player_id, seat_index) = room.add_player(username).map_err(|e| {
        format!("{{\"type\":\"error\",\"error\":\"{}\"}}", e)
    })?;

    conn.current_room = Some(room_id);
    conn.player_id = Some(player_id);

    drop(rooms);

    // Send join response to the joining client
    let join_msg = serde_json::to_string(&ServerMessage::LobbyJoined {
        room_id,
        player_id,
        seat_index,
    })
    .unwrap();

    // Broadcast updated player list to all in the room
    broadcast_lobby_update(room_id, state).await;

    Ok(Some(join_msg))
}

async fn handle_lobby_leave(
    conn: &mut ConnectionState,
    state: &AppState,
) -> Result<Option<String>, String> {
    let room_id = conn.current_room.take();
    let player_id = conn.player_id.take();

    if let (Some(rid), Some(pid)) = (room_id, player_id) {
        let mut rooms = state.rooms.write().await;
        if let Some(room) = rooms.get_mut(&rid) {
            room.remove_player(pid);
        }
        drop(rooms);
        broadcast_lobby_update(rid, state).await;
    }

    Ok(Some("{\"type\":\"lobby:left\"}".to_string()))
}

async fn handle_lobby_start(
    hidden_mode: bool,
    difficulty: BotDifficulty,
    conn: &mut ConnectionState,
    state: &AppState,
) -> Result<Option<String>, String> {
    let room_id = conn.current_room.ok_or_else(|| {
        "{\"type\":\"error\",\"error\":\"Not in a room\"}".to_string()
    })?;

    let mut rooms = state.rooms.write().await;
    let room = rooms.get_mut(&room_id).ok_or_else(|| {
        "{\"type\":\"error\",\"error\":\"Room not found\"}".to_string()
    })?;

    if !room.is_ready() {
        return Err("{\"type\":\"error\",\"error\":\"Room not ready (need 4 players)\"}".to_string());
    }

    room.hidden_mode = hidden_mode;
    room.difficulty = difficulty;
    room.table.deal();
    room.is_started = true;

    // Build state and broadcast to all
    let state_resp = crate::routes::build_table_state(&room.table);
    let game_msg = serde_json::to_string(&ServerMessage::GameState {
        state: state_resp,
        room_id,
    })
    .unwrap();

    // Broadcast via channel
    if let Some(tx) = state.room_broadcast.read().await.get(&room_id) {
        let _ = tx.send(game_msg);
    }

    drop(rooms);

    // Also send lobby:started
    let started_msg = serde_json::to_string(&ServerMessage::LobbyStarted { room_id }).unwrap();
    if let Some(tx) = state.room_broadcast.read().await.get(&room_id) {
        let _ = tx.send(started_msg);
    }

    Ok(Some("{\"type\":\"lobby:started\",\"ok\":true}".to_string()))
}

async fn handle_lobby_toggle_hidden(
    enabled: bool,
    conn: &mut ConnectionState,
    state: &AppState,
) -> Result<Option<String>, String> {
    let room_id = conn.current_room.ok_or_else(|| {
        "{\"type\":\"error\",\"error\":\"Not in a room\"}".to_string()
    })?;

    let mut rooms = state.rooms.write().await;
    if let Some(room) = rooms.get_mut(&room_id) {
        room.hidden_mode = enabled;
    }
    drop(rooms);

    broadcast_lobby_update(room_id, state).await;

    Ok(Some("{\"type\":\"lobby:hidden_toggled\",\"enabled\":true}".to_string()))
}

/// Broadcast lobby player list update to all connections in a room.
async fn broadcast_lobby_update(room_id: Uuid, state: &AppState) {
    let rooms = state.rooms.read().await;
    let room = match rooms.get(&room_id) {
        Some(r) => r,
        None => return,
    };

    let players: Vec<LobbyPlayerInfo> = room
        .sessions
        .values()
        .map(|s| LobbyPlayerInfo {
            name: s.player_name.clone(),
            seat_index: s.seat_index,
            is_bot: false,
        })
        .collect();

    let msg = serde_json::to_string(&ServerMessage::LobbyUpdate {
        players,
        hidden_mode: room.hidden_mode,
    })
    .unwrap();
    drop(rooms);

    if let Some(tx) = state.room_broadcast.read().await.get(&room_id) {
        let _ = tx.send(msg);
    }
}

// ── Chat ───────────────────────────────────────────────────────────────

async fn handle_chat_send(
    _player_id: String,
    text: String,
    conn: &mut ConnectionState,
    state: &AppState,
) -> Result<Option<String>, String> {
    tracing::info!(text, "handle_chat_send called");
    let room_id = conn.current_room.ok_or_else(|| {
        tracing::warn!("Chat send: not in a room");
        "{\"type\":\"error\",\"error\":\"Not in a room\"}".to_string()
    })?;
    let player_name = conn.username.as_deref().unwrap_or("Unknown");
    tracing::info!(%room_id, %player_name, text, "Chat send processing");

    let mut rooms = state.rooms.write().await;
    let room = rooms.get_mut(&room_id).ok_or_else(|| {
        "{\"type\":\"error\",\"error\":\"Room not found\"}".to_string()
    })?;

    let chat_msg = room.add_message(player_name, &text);
    drop(rooms);

    let msg = serde_json::to_string(&ServerMessage::ChatMessage {
        id: chat_msg.id,
        player_name: chat_msg.player_name,
        text: chat_msg.text,
        timestamp: chat_msg.timestamp,
    })
    .unwrap();

    if let Some(tx) = state.room_broadcast.read().await.get(&room_id) {
        let _ = tx.send(msg);
    }

    Ok(Some("{\"type\":\"chat:sent\",\"ok\":true}".to_string()))
}

// ── Game actions ───────────────────────────────────────────────────────

async fn handle_game_action(
    action_type: String,
    call: Option<serde_json::Value>,
    card: Option<serde_json::Value>,
    conn: &mut ConnectionState,
    state: &AppState,
) -> Result<Option<String>, String> {
    let room_id = conn.current_room.ok_or_else(|| {
        "{\"type\":\"error\",\"error\":\"Not in a room\"}".to_string()
    })?;

    // Check game type: single-player or multiplayer
    // For now, handle both via the same path
    let mut rooms = state.rooms.write().await;
    let room = rooms.get_mut(&room_id).ok_or_else(|| {
        "{\"type\":\"error\",\"error\":\"Room not found\"}".to_string()
    })?;

    let username = conn.username.as_deref().unwrap_or("");
    let human_seat = match room
        .sessions
        .values()
        .find(|s| s.player_name == username)
    {
        Some(s) => s.seat_index,
        None => {
            return Err(
                "{\"type\":\"error\",\"error\":\"You are not a player in this room\"}".to_string(),
            );
        }
    };

    let human_action = parse_human_action(&action_type, &call, &card)?;

    // Use the room's stored difficulty (set when the game started)
    let difficulty = room.difficulty;

    game_session::action_human_move(&mut room.table, human_seat, &human_action, difficulty)
        .map_err(|e| format!("{{\"type\":\"error\",\"error\":\"{}\"}}", e))?;

    let state_resp = crate::routes::build_table_state(&room.table);
    drop(rooms);

    // Broadcast updated state to all in the room
    let game_msg = serde_json::to_string(&ServerMessage::GameState {
        state: state_resp,
        room_id,
    })
    .unwrap();

    if let Some(tx) = state.room_broadcast.read().await.get(&room_id) {
        let _ = tx.send(game_msg.clone());
    }

    Ok(Some(game_msg))
}

/// Parse a human action from WebSocket message fields.
fn parse_human_action(
    action_type: &str,
    call: &Option<serde_json::Value>,
    card: &Option<serde_json::Value>,
) -> Result<HumanAction, String> {
    match action_type {
        "bid" => {
            let call_val = call.clone().ok_or("Missing 'call' for bid")?;
            let api_call: game_core::Call =
                serde_json::from_value(call_val).map_err(|e| format!("Invalid call: {}", e))?;
            Ok(HumanAction::Call(api_call))
        }
        "play" => {
            let card_val = card.clone().ok_or("Missing 'card' for play")?;
            let api_card: game_core::Card =
                serde_json::from_value(card_val).map_err(|e| format!("Invalid card: {}", e))?;
            Ok(HumanAction::PlayCard(api_card))
        }
        "selectPartner" => {
            let card_val = card.clone().ok_or("Missing 'card' for selectPartner")?;
            let api_card: game_core::Card =
                serde_json::from_value(card_val).map_err(|e| format!("Invalid card: {}", e))?;
            Ok(HumanAction::SelectPartner(api_card))
        }
        _ => Err(format!("Unknown action type: {}", action_type)),
    }
}

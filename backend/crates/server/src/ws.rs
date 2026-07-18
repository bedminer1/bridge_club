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
use tokio::{
    sync::broadcast,
    time::{timeout, Duration},
};
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
    LobbyJoin {
        #[serde(rename = "roomId")]
        room_id: String,
    },
    #[serde(rename = "lobby:leave")]
    LobbyLeave {
        #[serde(rename = "playerId")]
        _player_id: String,
    },
    #[serde(rename = "lobby:start")]
    LobbyStart {
        #[serde(rename = "hiddenMode")]
        hidden_mode: Option<bool>,
        difficulty: Option<String>,
    },
    #[serde(rename = "lobby:toggle_hidden")]
    LobbyToggleHidden { enabled: bool },
    #[serde(rename = "game:action")]
    GameAction {
        #[serde(rename = "actionType")]
        action_type: String,
        call: Option<serde_json::Value>,
        card: Option<serde_json::Value>,
    },
    #[serde(rename = "chat:send")]
    ChatSend {
        #[serde(rename = "playerId")]
        player_id: String,
        text: String,
    },
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
        players: Vec<LobbyPlayerInfo>,
        #[serde(rename = "hiddenMode")]
        hidden_mode: bool,
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
        state: TableStateResponse,
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
    /// Unique ID for this websocket connection.
    connection_id: Uuid,
    /// Authenticated user ID (None before auth).
    user_id: Option<i64>,
    username: Option<String>,
    /// Current room the connection is subscribed to.
    current_room: Option<Uuid>,
    /// Player ID in the current room (for lobby operations).
    player_id: Option<Uuid>,
    /// If set, the main loop should run a lobby update broadcast after
    /// the connection has subscribed to the room's broadcast channel.
    pending_broadcast_update: Option<Uuid>,
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
    let connection_id = Uuid::new_v4();

    tracing::info!(%connection_id, "WebSocket connection opened");

    let mut conn = ConnectionState {
        connection_id,
        user_id: None,
        username: None,
        current_room: None,
        player_id: None,
        pending_broadcast_update: None,
    };

    // Broadcast receiver handle — dropped when leaving a room.
    let mut broadcast_rx: Option<broadcast::Receiver<String>> = None;
    let mut subscribed_room: Option<Uuid> = None;

    // Main message loop
    loop {
        // Poll websocket input with a short timeout so room broadcasts are
        // forwarded even when no client messages are arriving.
        if let Ok(msg) = timeout(Duration::from_millis(20), receiver.next()).await {
            match msg {
                Some(Ok(Message::Text(text))) => {
                    tracing::info!(raw_msg = %text, "WS received text message");
                    let response = handle_client_message(&text, &mut conn, &state).await;

                    // Keep websocket room subscription in sync with current_room.
                    sync_room_subscription(
                        &state,
                        &mut broadcast_rx,
                        &mut subscribed_room,
                        conn.current_room,
                        conn.connection_id,
                    ).await;

                    // If the handler requested a pending lobby broadcast (e.g.
                    // join should subscribe first), run it now.
                    if let Some(room_to_broadcast) = conn.pending_broadcast_update.take() {
                        broadcast_lobby_update(room_to_broadcast, &state).await;
                    }

                    // Now send handler response.
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
                }
                Some(Ok(Message::Close(_))) | None => break,
                _ => {}
            }
        }

        // Drain room broadcasts on every loop tick.
        if let Some(ref mut rx) = broadcast_rx {
            let mut should_close = false;
            loop {
                match rx.try_recv() {
                    Ok(msg) => {
                        if sender.send(Message::Text(msg.into())).await.is_err() {
                            should_close = true;
                            break;
                        }
                    }
                    Err(broadcast::error::TryRecvError::Empty) => break,
                    Err(broadcast::error::TryRecvError::Lagged(n)) => {
                        tracing::warn!("Broadcast lagged by {} messages", n);
                    }
                    Err(broadcast::error::TryRecvError::Closed) => break,
                }
            }
            if should_close {
                break;
            }
        }
    }

    // Cleanup: remove from broadcast channel
    if let Some(room_id) = subscribed_room {
        unregister_listener(&state, room_id, conn.connection_id).await;
        let rooms = state.rooms.write().await;
        if let Some(_room) = rooms.get(&room_id) {
            if let Some(pid) = conn.player_id {
                tracing::info!(
                    "Player websocket disconnected from room {} (player {})",
                    room_id,
                    pid
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
        let err = format!("{{\"type\":\"error\",\"error\":\"Invalid message: {}\"}}", e);
        tracing::warn!(raw = %text, error = %e, "WS deserialization failed");
        err
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
    let validated = match auth::validate_session(&state.db, &token).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Auth error: {}", e);
            return Err("{\"type\":\"auth:error\",\"error\":\"Internal error\"}".to_string());
        }
    };

    match validated {
        Some((user, _session)) => {
            conn.user_id = Some(user.id);
            conn.username = Some(user.username.clone());

            if let Some((room_id, player_id, seat_index, is_started)) =
                find_existing_player_session(&state.rooms, &user.username).await
            {
                conn.current_room = Some(room_id);
                conn.player_id = Some(player_id);
                tracing::info!(
                    %room_id,
                    %player_id,
                    seat_index,
                    is_started,
                    username = %user.username,
                    "Restored websocket room binding from existing player session"
                );
            }

            Ok(Some(
                serde_json::to_string(&ServerMessage::AuthOk {
                    user_id: user.id,
                    username: user.username,
                })
                .unwrap(),
            ))
        }
        None => Err(
            "{\"type\":\"auth:error\",\"error\":\"Invalid or expired session\"}".to_string(),
        ),
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

    let (player_id, seat_index, should_broadcast_lobby_update) =
        if let Some((existing_player_id, existing_session)) = room
            .sessions
            .iter()
            .find(|(_, s)| s.player_name == username)
        {
            tracing::info!(
                %room_id,
                username,
                player_id = %existing_player_id,
                seat_index = existing_session.seat_index,
                "Reusing existing room session for lobby join"
            );
            (*existing_player_id, existing_session.seat_index, false)
        } else if room.is_started {
            return Err(
                "{\"type\":\"error\",\"error\":\"Game already started and user is not seated\"}".to_string(),
            );
        } else {
            let (new_player_id, new_seat_index) = room.add_player(username).map_err(|e| {
                format!("{{\"type\":\"error\",\"error\":\"{}\"}}", e)
            })?;
            (new_player_id, new_seat_index, true)
        };

    conn.current_room = Some(room_id);
    conn.player_id = Some(player_id);

    let players: Vec<LobbyPlayerInfo> = room
        .sessions
        .values()
        .map(|s| LobbyPlayerInfo {
            name: s.player_name.clone(),
            seat_index: s.seat_index,
            is_bot: s.player_name.starts_with("Bot-"),
        })
        .collect();
    let hidden_mode = room.hidden_mode;

    if !state.room_broadcast.read().await.contains_key(&room_id) {
        let (tx, _) = broadcast::channel(256);
        state.room_broadcast.write().await.insert(room_id, tx);
        tracing::info!(%room_id, "Created missing broadcast channel for joined room");
    }

    drop(rooms);

    // Send join response to the joining client
    let join_msg = serde_json::to_string(&ServerMessage::LobbyJoined {
        room_id,
        player_id,
        seat_index,
        players,
        hidden_mode,
    })
    .unwrap();

    // Only broadcast lobby roster when we actually changed lobby membership.
    if should_broadcast_lobby_update {
        conn.pending_broadcast_update = Some(room_id);
    }

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

    // Fill remaining slots with bots
    let bot_names = ["Bot-Alpha", "Bot-Beta", "Bot-Gamma"];
    let mut next_bot = 0;
    while room.sessions.len() < 4 {
        let bot_name = bot_names[next_bot % bot_names.len()];
        room.add_player(bot_name).map_err(|e| {
            format!("{{\"type\":\"error\",\"error\":\"{}\"}}", e)
        })?;
        next_bot += 1;
    }

    if !room.is_ready() {
        return Err("{\"type\":\"error\",\"error\":\"Room not ready (need 4 players)\"}".to_string());
    }

    room.hidden_mode = hidden_mode;
    room.difficulty = difficulty;
    room.table.deal();
    room.is_started = true;

    // Auto-advance any initial bot turns (before first human move)
    let username = conn.username.as_deref().unwrap_or("");
    let human_seat = room
        .sessions
        .values()
        .find(|s| s.player_name == username)
        .map(|s| s.seat_index)
        .unwrap_or(0);
    use game_core::GamePhase;
    loop {
        let current = room.table.current_player_index();
        if room.table.phase == GamePhase::Finished {
            break;
        }
        if !room.table.players[current].name.starts_with("Bot-") {
            break;
        }
        match game_session::advance_one_turn(&mut room.table, human_seat, difficulty) {
            Ok(true) => {}
            _ => break,
        }
    }

    // Build state and broadcast to all
    let state_resp = crate::routes::build_table_state(&room.table);
    drop(rooms);

    // Send lobby:started first with the initial state so clients can switch
    // to online mode before the follow-up game:state broadcast arrives.
    let started_msg = serde_json::to_string(&ServerMessage::LobbyStarted {
        room_id,
        state: state_resp.clone(),
    })
    .unwrap();
    if let Some(tx) = state.room_broadcast.read().await.get(&room_id) {
        match tx.send(started_msg) {
            Ok(subscriber_count) => {
                tracing::info!(%room_id, subscriber_count, "Broadcasted lobby:started to room listeners");
            }
            Err(err) => {
                tracing::warn!(%room_id, error = %err, "Failed to broadcast lobby:started to room listeners");
            }
        }
    } else {
        tracing::warn!(%room_id, "No broadcast sender available for lobby:started");
    }

    let _game_msg = broadcast_game_state(room_id, &state_resp, state, "lobby:start").await;

    Ok(Some(
        serde_json::to_string(&ServerMessage::LobbyStarted {
            room_id,
            state: state_resp.clone(),
        })
        .unwrap(),
    ))
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
    let (listener_ids, listener_count) = snapshot_listeners(&state, room_id).await;
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
            is_bot: s.player_name.starts_with("Bot-"),
        })
        .collect();

    let msg = serde_json::to_string(&ServerMessage::LobbyUpdate {
        players,
        hidden_mode: room.hidden_mode,
    })
    .unwrap();
    drop(rooms);

    if let Some(tx) = state.room_broadcast.read().await.get(&room_id) {
        match tx.send(msg) {
            Ok(subscriber_count) => {
                tracing::info!(%room_id, listener_count, ?listener_ids, subscriber_count, "Broadcasted lobby update to room listeners");
            }
            Err(err) => {
                tracing::warn!(%room_id, listener_count, ?listener_ids, error = %err, "Failed to broadcast lobby update to room listeners");
            }
        }
    } else {
        tracing::warn!(%room_id, "No broadcast sender available for lobby update");
    }
}

async fn broadcast_game_state(
    room_id: Uuid,
    state_resp: &TableStateResponse,
    state: &AppState,
    broadcast_source: &str,
) -> String {
    let (listener_ids, listener_count) = snapshot_listeners(&state, room_id).await;
    let game_msg = serde_json::to_string(&ServerMessage::GameState {
        state: state_resp.clone(),
        room_id,
    })
    .unwrap();

    if let Some(tx) = state.room_broadcast.read().await.get(&room_id) {
        match tx.send(game_msg.clone()) {
            Ok(subscriber_count) => {
                tracing::info!(%room_id, broadcast_source, listener_count, ?listener_ids, subscriber_count, "Broadcasted game state to room listeners");
            }
            Err(err) => {
                tracing::warn!(%room_id, broadcast_source, listener_count, ?listener_ids, error = %err, "Failed to broadcast game state to room listeners");
            }
        }
    } else {
        tracing::warn!(%room_id, broadcast_source, "No broadcast sender available for game state");
    }

    game_msg
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

    let (listener_ids, listener_count) = snapshot_listeners(&state, room_id).await;
    if let Some(tx) = state.room_broadcast.read().await.get(&room_id) {
        match tx.send(msg) {
            Ok(subscriber_count) => {
                tracing::info!(%room_id, listener_count, ?listener_ids, subscriber_count, "Broadcasted chat message to room listeners");
            }
            Err(err) => {
                tracing::warn!(%room_id, listener_count, ?listener_ids, error = %err, "Failed to broadcast chat message to room listeners");
            }
        }
    } else {
        tracing::warn!(%room_id, "No broadcast sender available for chat message");
    }

    Ok(Some("{\"type\":\"chat:sent\",\"ok\":true}".to_string()))
}

async fn find_existing_player_session(
    rooms_lock: &tokio::sync::RwLock<std::collections::HashMap<Uuid, crate::session::GameRoom>>,
    username: &str,
) -> Option<(Uuid, Uuid, usize, bool)> {
    let rooms = rooms_lock.read().await;
    for (room_id, room) in rooms.iter() {
        if let Some((player_id, session)) = room
            .sessions
            .iter()
            .find(|(_, session)| session.player_name == username)
        {
            return Some((*room_id, *player_id, session.seat_index, room.is_started));
        }
    }
    None
}

async fn sync_room_subscription(
    state: &AppState,
    broadcast_rx: &mut Option<broadcast::Receiver<String>>,
    subscribed_room: &mut Option<Uuid>,
    desired_room: Option<Uuid>,
    connection_id: Uuid,
) {
    if *subscribed_room == desired_room {
        return;
    }

    if let Some(old_room) = subscribed_room.take() {
        unregister_listener(state, old_room, connection_id).await;
        *broadcast_rx = None;
    }

    if let Some(room_id) = desired_room {
        let next_rx = state
            .room_broadcast
            .read()
            .await
            .get(&room_id)
            .map(|tx| tx.subscribe());

        if let Some(rx) = next_rx {
            *broadcast_rx = Some(rx);
            *subscribed_room = Some(room_id);
            register_listener(state, room_id, connection_id).await;
        } else {
            tracing::warn!(
                %room_id,
                %connection_id,
                "Could not subscribe websocket: room broadcast sender missing"
            );
        }
    }
}

async fn register_listener(state: &AppState, room_id: Uuid, connection_id: Uuid) {
    let mut listeners = state.room_listeners.write().await;
    let room_listeners = listeners.entry(room_id).or_default();
    room_listeners.insert(connection_id);
    let listener_count = room_listeners.len();
    let listener_ids: Vec<Uuid> = room_listeners.iter().copied().collect();
    tracing::info!(%room_id, %connection_id, listener_count, ?listener_ids, "WebSocket subscribed to room listeners");
}

async fn unregister_listener(state: &AppState, room_id: Uuid, connection_id: Uuid) {
    let mut listeners = state.room_listeners.write().await;
    if let Some(room_listeners) = listeners.get_mut(&room_id) {
        room_listeners.remove(&connection_id);
        let listener_count = room_listeners.len();
        let listener_ids: Vec<Uuid> = room_listeners.iter().copied().collect();
        tracing::info!(%room_id, %connection_id, listener_count, ?listener_ids, "WebSocket unsubscribed from room listeners");
        if room_listeners.is_empty() {
            listeners.remove(&room_id);
        }
    }
}

async fn snapshot_listeners(state: &AppState, room_id: Uuid) -> (Vec<Uuid>, usize) {
    let listeners = state.room_listeners.read().await;
    let listener_ids: Vec<Uuid> = listeners
        .get(&room_id)
        .map(|room_listeners| room_listeners.iter().copied().collect())
        .unwrap_or_default();
    let listener_count = listener_ids.len();
    (listener_ids, listener_count)
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

    // Auto-advance consecutive bot turns (no more HTTP polling)
    use game_core::GamePhase;
    loop {
        let current = room.table.current_player_index();
        if room.table.phase == GamePhase::Finished {
            break;
        }
        let player_name = &room.table.players[current].name;
        if !player_name.starts_with("Bot-") {
            break;
        }
        match game_session::advance_one_turn(&mut room.table, human_seat, difficulty) {
            Ok(true) => {} // continue to next bot
            _ => break,
        }
    }

    let state_resp = crate::routes::build_table_state(&room.table);
    drop(rooms);

    // Broadcast updated state to all in the room
    let game_msg = broadcast_game_state(room_id, &state_resp, state, "game:action").await;

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

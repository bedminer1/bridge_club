use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::session::AppState;
use game_core::Call;

// ── Request / Response types ──────────────────────────────────────────────

#[derive(Serialize)]
pub struct CreateRoomResponse {
    pub room_id: Uuid,
}

#[derive(Deserialize)]
pub struct JoinRoomRequest {
    pub player_name: String,
}

#[derive(Serialize)]
pub struct JoinRoomResponse {
    pub player_id: Uuid,
    pub seat_index: usize,
    pub room_id: Uuid,
}

#[derive(Deserialize)]
pub struct MakeCallRequest {
    pub player_id: Uuid,
    pub call: Call,
}

#[derive(Deserialize)]
pub struct SelectPartnerRequest {
    pub player_id: Uuid,
    pub card: game_core::Card,
}

#[derive(Serialize)]
pub struct TableStateResponse {
    pub phase: String,
    pub hands: Vec<String>,
    pub current_player: usize,
    pub bet_size: u8,
    pub trump_suit: Option<String>,
    pub bet_winner: Option<usize>,
    pub partner_idx: Option<usize>,
    pub sets_won: Vec<u8>,
    pub completed_set_count: usize,
    pub is_finished: bool,
}

// ── Routes ────────────────────────────────────────────────────────────────

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(|| async { "Bridge Club API v0.1" }))
        .route("/room", post(create_room))
        .route("/room/{room_id}/join", post(join_room))
        .route("/room/{room_id}/leave/{player_id}", post(leave_room))
        .route("/room/{room_id}/start", post(start_game))
        .route("/room/{room_id}/state", get(get_table_state))
        .route("/room/{room_id}/call", post(make_call))
        .route("/room/{room_id}/select-partner", post(select_partner))
        .route("/room/{room_id}/play/{player_id}", post(play_card))
        .with_state(state)
}

// ── Handlers ──────────────────────────────────────────────────────────────

async fn create_room(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let room = crate::session::GameRoom::new();
    let room_id = room.room_id;

    let mut rooms = state.rooms.write().await;
    rooms.insert(room_id, room);

    tracing::info!("Created room {}", room_id);
    (StatusCode::CREATED, Json(CreateRoomResponse { room_id }))
}

async fn join_room(
    State(state): State<AppState>,
    Path(room_id): Path<Uuid>,
    Json(payload): Json<JoinRoomRequest>,
) -> impl IntoResponse {
    let mut rooms = state.rooms.write().await;
    let room = match rooms.get_mut(&room_id) {
        Some(r) => r,
        None => return (StatusCode::NOT_FOUND, Json(JoinRoomResponse {
            player_id: Uuid::nil(),
            seat_index: 0,
            room_id,
        })),
    };

    match room.add_player(&payload.player_name) {
        Ok((player_id, seat_index)) => {
            tracing::info!(
                "Player '{}' joined room {} as seat {}",
                payload.player_name, room_id, seat_index
            );
            (StatusCode::OK, Json(JoinRoomResponse { player_id, seat_index, room_id }))
        }
        Err(e) => {
            tracing::warn!("Failed to join room {}: {}", room_id, e);
            (StatusCode::BAD_REQUEST, Json(JoinRoomResponse {
                player_id: Uuid::nil(),
                seat_index: 0,
                room_id,
            }))
        }
    }
}

async fn leave_room(
    State(state): State<AppState>,
    Path((room_id, player_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    let mut rooms = state.rooms.write().await;
    let room = match rooms.get_mut(&room_id) {
        Some(r) => r,
        None => return StatusCode::NOT_FOUND,
    };

    room.remove_player(player_id);

    // Clean up empty rooms
    if room.player_count() == 0 {
        rooms.remove(&room_id);
        tracing::info!("Removed empty room {}", room_id);
    }

    StatusCode::OK
}

async fn start_game(
    State(state): State<AppState>,
    Path(room_id): Path<Uuid>,
) -> impl IntoResponse {
    let mut rooms = state.rooms.write().await;
    let room = match rooms.get_mut(&room_id) {
        Some(r) => r,
        None => return StatusCode::NOT_FOUND,
    };

    if !room.is_ready() {
        return StatusCode::BAD_REQUEST;
    }

    room.table.deal();
    room.is_started = true;
    tracing::info!("Game started in room {}", room_id);
    StatusCode::OK
}

async fn get_table_state(
    State(state): State<AppState>,
    Path(room_id): Path<Uuid>,
) -> impl IntoResponse {
    let rooms = state.rooms.read().await;
    let room = match rooms.get(&room_id) {
        Some(r) => r,
        None => return (StatusCode::NOT_FOUND, Json(TableStateResponse {
            phase: String::new(),
            hands: Vec::new(),
            current_player: 0,
            bet_size: 0,
            trump_suit: None,
            bet_winner: None,
            partner_idx: None,
            sets_won: Vec::new(),
            completed_set_count: 0,
            is_finished: false,
        })),
    };

    let hands: Vec<String> = room.table.players
        .iter()
        .map(|p| {
            p.hand.iter()
                .map(|c| c.to_ascii_string())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect();

    let phase = format!("{:?}", room.table.phase);
    let current_player = room.table.current_player_index();
    let trump_suit = room.table.trump_suit.map(|s| s.to_string());
    let sets_won = room.table.sets_won.to_vec();
    let is_finished = room.table.phase == game_core::GamePhase::Finished;

    (
        StatusCode::OK,
        Json(TableStateResponse {
            phase,
            hands,
            current_player,
            bet_size: room.table.bet_size,
            trump_suit,
            bet_winner: room.table.bet_winner,
            partner_idx: room.table.partner_idx,
            sets_won,
            completed_set_count: room.table.completed_sets.len(),
            is_finished,
        }),
    )
}

async fn make_call(
    State(state): State<AppState>,
    Path(room_id): Path<Uuid>,
    Json(payload): Json<MakeCallRequest>,
) -> impl IntoResponse {
    let mut rooms = state.rooms.write().await;
    let room = match rooms.get_mut(&room_id) {
        Some(r) => r,
        None => return StatusCode::NOT_FOUND,
    };

    // Verify player is in this room
    if !room.sessions.contains_key(&payload.player_id) {
        return StatusCode::FORBIDDEN;
    }

    match room.table.make_call(payload.call) {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            tracing::warn!("Call rejected in room {}: {}", room_id, e);
            StatusCode::BAD_REQUEST
        }
    }
}

async fn select_partner(
    State(state): State<AppState>,
    Path(room_id): Path<Uuid>,
    Json(payload): Json<SelectPartnerRequest>,
) -> impl IntoResponse {
    let mut rooms = state.rooms.write().await;
    let room = match rooms.get_mut(&room_id) {
        Some(r) => r,
        None => return StatusCode::NOT_FOUND,
    };

    // Verify player is in this room
    if !room.sessions.contains_key(&payload.player_id) {
        return StatusCode::FORBIDDEN;
    }

    match room.table.select_partner(payload.card) {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            tracing::warn!("Partner selection rejected in room {}: {}", room_id, e);
            StatusCode::BAD_REQUEST
        }
    }
}

async fn play_card(
    State(state): State<AppState>,
    Path((room_id, player_id)): Path<(Uuid, Uuid)>,
    Json(card): Json<game_core::Card>,
) -> impl IntoResponse {
    let mut rooms = state.rooms.write().await;
    let room = match rooms.get_mut(&room_id) {
        Some(r) => r,
        None => return StatusCode::NOT_FOUND,
    };

    // Verify the player is in this room
    if !room.sessions.contains_key(&player_id) {
        return StatusCode::FORBIDDEN;
    }

    match room.table.play_card(card) {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            tracing::warn!("Card play rejected in room {}: {}", room_id, e);
            StatusCode::BAD_REQUEST
        }
    }
}

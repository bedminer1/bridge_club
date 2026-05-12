use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
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

#[derive(Serialize)]
pub struct TableStateResponse {
    pub phase: String,
    pub hands: Vec<String>,
    pub current_player: usize,
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
        .route("/room/{room_id}/play/{player_id}", post(play_card))
        .with_state(state)
}

// ── Handlers (stubs — return 501 Not Implemented) ─────────────────────────

async fn create_room(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    // TODO: create a new GameRoom, insert into state, return room_id
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(CreateRoomResponse {
            room_id: Uuid::nil(),
        }),
    )
}

async fn join_room(
    State(_state): State<AppState>,
    Path(_room_id): Path<Uuid>,
    Json(_payload): Json<JoinRoomRequest>,
) -> impl IntoResponse {
    // TODO: look up room, add player, return JoinRoomResponse
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(JoinRoomResponse {
            player_id: Uuid::nil(),
            seat_index: 0,
            room_id: Uuid::nil(),
        }),
    )
}

async fn leave_room(
    State(_state): State<AppState>,
    Path((_room_id, _player_id)): Path<(Uuid, Uuid)>,
) -> impl IntoResponse {
    // TODO: remove player from room
    StatusCode::NOT_IMPLEMENTED
}

async fn start_game(
    State(_state): State<AppState>,
    Path(_room_id): Path<Uuid>,
) -> impl IntoResponse {
    // TODO: verify room is ready, call table.deal()
    StatusCode::NOT_IMPLEMENTED
}

async fn get_table_state(
    State(_state): State<AppState>,
    Path(_room_id): Path<Uuid>,
) -> impl IntoResponse {
    // TODO: look up room, extract current table state
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(TableStateResponse {
            phase: String::new(),
            hands: Vec::new(),
            current_player: 0,
        }),
    )
}

async fn make_call(
    State(_state): State<AppState>,
    Path(_room_id): Path<Uuid>,
    Json(_payload): Json<MakeCallRequest>,
) -> impl IntoResponse {
    // TODO: authenticate player_id, forward call to table.make_call()
    StatusCode::NOT_IMPLEMENTED
}

async fn play_card(
    State(_state): State<AppState>,
    Path((_room_id, _player_id)): Path<(Uuid, Uuid)>,
    Json(_card): Json<game_core::Card>,
) -> impl IntoResponse {
    // TODO: authenticate player, forward to table.play_card()
    StatusCode::NOT_IMPLEMENTED
}

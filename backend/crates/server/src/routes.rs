use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth;
use crate::bot::BotDifficulty;
use crate::db::{DbPool, MatchRow, UserRow};
use crate::game_session::{self, HumanAction};
use crate::session::AppState;
use game_core::Call;

// ── Request / Response types ──────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoomResponse {
    pub room_id: Uuid,
    pub player_id: Uuid,
    pub seat_index: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomInfoResponse {
    pub room_id: Uuid,
    pub is_started: bool,
    pub phase: String,
    pub players: Vec<RoomPlayerInfo>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomPlayerInfo {
    pub name: String,
    pub seat_index: usize,
    pub is_bot: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
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
    pub current_trick_cards: Vec<game_core::Card>,
    pub current_trick_start_player: usize,
    pub previous_trick_cards: Vec<game_core::Card>,
    pub previous_trick_winner: Option<usize>,
    pub previous_trick_start_player: usize,
    pub call_history: Vec<game_core::Call>,
    pub call_history_start_player: usize,
    pub partner_card: Option<game_core::Card>,
    pub trump_played: bool,
    pub lead_suit: Option<String>,
    pub completed_sets: Vec<game_core::Set>,
}

// ── Auth request / response types ─────────────────────────────────────────

#[derive(Deserialize)]
pub struct SignupRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct SessionResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<UserInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
}

#[derive(Serialize)]
pub struct SuccessResponse {
    pub ok: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveMatchRequest {
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
}

#[derive(Serialize)]
pub struct MatchesResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matches: Option<Vec<MatchRow>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Deserialize)]
pub struct SessionQuery {
    pub token: Option<String>,
}

// ── Single-player game request / response types ────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewGameRequest {
    pub difficulty: String, // "Easy" or "Medium"
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewGameResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<TableStateResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameActionRequest {
    #[serde(rename = "type")]
    pub action_type: String, // "bid", "play", "selectPartner"
    pub call: Option<Call>,
    pub card: Option<game_core::Card>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameActionResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<TableStateResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// ── Helper: extract session token from headers or query ───────────────────

fn extract_token(headers: &HeaderMap, query: Option<&SessionQuery>) -> Option<String> {
    // Try header first: X-Session-Token
    if let Some(val) = headers.get("X-Session-Token") {
        if let Ok(s) = val.to_str() {
            if !s.is_empty() {
                return Some(s.to_string());
            }
        }
    }
    // Fall back to query param ?token=...
    if let Some(q) = query {
        if let Some(t) = &q.token {
            if !t.is_empty() {
                return Some(t.clone());
            }
        }
    }
    None
}

/// Validate the session from headers/query, returning the user row on success.
async fn require_user(
    pool: &DbPool,
    headers: &HeaderMap,
    query: Option<&SessionQuery>,
) -> Result<UserRow, (StatusCode, Json<AuthResponse>)> {
    let token = match extract_token(headers, query) {
        Some(t) => t,
        None => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(AuthResponse {
                    ok: false,
                    token: None,
                    user_id: None,
                    username: None,
                    error: Some("Missing session token".to_string()),
                }),
            ));
        }
    };

    match auth::validate_session(pool, &token).await {
        Ok(Some((user, _session))) => Ok(user),
        Ok(None) => Err((
            StatusCode::UNAUTHORIZED,
            Json(AuthResponse {
                ok: false,
                token: None,
                user_id: None,
                username: None,
                error: Some("Invalid or expired session".to_string()),
            }),
        )),
        Err(e) => {
            tracing::error!("Session validation error: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthResponse {
                    ok: false,
                    token: None,
                    user_id: None,
                    username: None,
                    error: Some("Internal server error".to_string()),
                }),
            ))
        }
    }
}

// ── Routes ────────────────────────────────────────────────────────────────

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(|| async { "Bridge Club API v0.1" }))
        // Game room routes
        .route("/api/rooms", post(create_room))
        .route("/api/rooms/{room_id}/join", post(join_room))
        .route("/api/rooms/{room_id}/leave/{player_id}", post(leave_room))
        .route("/api/rooms/{room_id}/start", post(start_game))
        .route("/api/rooms/{room_id}/info", get(get_room_info))
        .route("/room/{room_id}/state", get(get_table_state))
        .route("/room/{room_id}/call", post(make_call))
        .route("/room/{room_id}/select-partner", post(select_partner))
        .route("/room/{room_id}/play/{player_id}", post(play_card))
        // Auth routes
        .route("/api/auth/signup", post(signup))
        .route("/api/auth/login", post(login))
        .route("/api/auth/session", get(get_session))
        .route("/api/auth/logout", post(logout))
        // Match history routes
        .route("/api/matches", get(get_matches).post(save_match))
        // Single-player game routes
        .route("/api/game/new", post(create_single_player_game))
        .route("/api/game/{room_id}/action", post(single_player_action))
        .route("/api/game/{room_id}/advance", post(advance_game))
        .with_state(state)
}

// ── Auth Handlers ─────────────────────────────────────────────────────────

async fn signup(
    State(state): State<AppState>,
    Json(payload): Json<SignupRequest>,
) -> impl IntoResponse {
    if payload.username.is_empty() || payload.password.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(AuthResponse {
                ok: false,
                token: None,
                user_id: None,
                username: None,
                error: Some("Username and password are required".to_string()),
            }),
        );
    }

    let conn = match state.db.conn().await {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("DB connection error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthResponse {
                    ok: false,
                    token: None,
                    user_id: None,
                    username: None,
                    error: Some("Internal server error".to_string()),
                }),
            );
        }
    };

    // Check if username already exists
    let existing = conn
        .query(
            "SELECT id FROM users WHERE username = ?1",
            libsql::params![payload.username.clone()],
        )
        .await;

    match existing {
        Ok(mut rows) => {
            if let Ok(Some(_)) = rows.next().await {
                return (
                    StatusCode::CONFLICT,
                    Json(AuthResponse {
                        ok: false,
                        token: None,
                        user_id: None,
                        username: None,
                        error: Some("Username already taken".to_string()),
                    }),
                );
            }
        }
        Err(e) => {
            tracing::error!("DB query error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthResponse {
                    ok: false,
                    token: None,
                    user_id: None,
                    username: None,
                    error: Some("Internal server error".to_string()),
                }),
            );
        }
    }

    let password_hash = auth::hash_password(&payload.password);

    // Insert the new user
    let insert = conn
        .execute(
            "INSERT INTO users (username, password) VALUES (?1, ?2)",
            libsql::params![payload.username.clone(), password_hash],
        )
        .await;

    let user_id = match insert {
        Ok(_) => {
            // Get the inserted user's ID
            let mut rows = conn
                .query(
                    "SELECT id FROM users WHERE username = ?1",
                    libsql::params![payload.username.clone()],
                )
                .await
                .unwrap();

            match rows.next().await.unwrap() {
                Some(row) => row.get::<i64>(0).unwrap(),
                None => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(AuthResponse {
                            ok: false,
                            token: None,
                            user_id: None,
                            username: None,
                            error: Some("Failed to retrieve user".to_string()),
                        }),
                    );
                }
            }
        }
        Err(e) => {
            tracing::error!("User insert error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthResponse {
                    ok: false,
                    token: None,
                    user_id: None,
                    username: None,
                    error: Some("Failed to create user".to_string()),
                }),
            );
        }
    };

    // Create session
    match auth::create_session(&state.db, user_id).await {
        Ok((_session_id, token)) => {
            tracing::info!("User '{}' signed up (id={})", payload.username, user_id);
            (
                StatusCode::CREATED,
                Json(AuthResponse {
                    ok: true,
                    token: Some(token),
                    user_id: Some(user_id),
                    username: Some(payload.username),
                    error: None,
                }),
            )
        }
        Err(e) => {
            tracing::error!("Session creation error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthResponse {
                    ok: false,
                    token: None,
                    user_id: None,
                    username: None,
                    error: Some("Failed to create session".to_string()),
                }),
            )
        }
    }
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> impl IntoResponse {
    if payload.username.is_empty() || payload.password.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(AuthResponse {
                ok: false,
                token: None,
                user_id: None,
                username: None,
                error: Some("Username and password are required".to_string()),
            }),
        );
    }

    let conn = match state.db.conn().await {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("DB connection error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthResponse {
                    ok: false,
                    token: None,
                    user_id: None,
                    username: None,
                    error: Some("Internal server error".to_string()),
                }),
            );
        }
    };

    let password_hash = auth::hash_password(&payload.password);

    let mut rows = match conn
        .query(
            "SELECT id, username, password FROM users WHERE username = ?1",
            libsql::params![payload.username.clone()],
        )
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("DB query error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthResponse {
                    ok: false,
                    token: None,
                    user_id: None,
                    username: None,
                    error: Some("Internal server error".to_string()),
                }),
            );
        }
    };

    let user = match rows.next().await {
        Ok(Some(row)) => {
            let db_password: String = row.get::<String>(2).unwrap();
            if db_password != password_hash {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(AuthResponse {
                        ok: false,
                        token: None,
                        user_id: None,
                        username: None,
                        error: Some("Invalid username or password".to_string()),
                    }),
                );
            }
            UserRow {
                id: row.get::<i64>(0).unwrap(),
                username: row.get::<String>(1).unwrap(),
                password: db_password,
            }
        }
        Ok(None) => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(AuthResponse {
                    ok: false,
                    token: None,
                    user_id: None,
                    username: None,
                    error: Some("Invalid username or password".to_string()),
                }),
            );
        }
        Err(e) => {
            tracing::error!("DB query error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthResponse {
                    ok: false,
                    token: None,
                    user_id: None,
                    username: None,
                    error: Some("Internal server error".to_string()),
                }),
            );
        }
    };

    // Create session
    match auth::create_session(&state.db, user.id).await {
        Ok((_session_id, token)) => {
            tracing::info!("User '{}' logged in (id={})", user.username, user.id);
            (
                StatusCode::OK,
                Json(AuthResponse {
                    ok: true,
                    token: Some(token),
                    user_id: Some(user.id),
                    username: Some(user.username),
                    error: None,
                }),
            )
        }
        Err(e) => {
            tracing::error!("Session creation error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthResponse {
                    ok: false,
                    token: None,
                    user_id: None,
                    username: None,
                    error: Some("Failed to create session".to_string()),
                }),
            )
        }
    }
}

async fn get_session(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<SessionQuery>,
) -> impl IntoResponse {
    match require_user(&state.db, &headers, Some(&query)).await {
        Ok(user) => (
            StatusCode::OK,
            Json(SessionResponse {
                ok: true,
                user: Some(UserInfo {
                    id: user.id,
                    username: user.username,
                }),
                error: None,
            }),
        ),
        Err((status, json)) => (
            status,
            Json(SessionResponse {
                ok: false,
                user: None,
                error: Some(json.0.error.unwrap_or_else(|| "Unauthorized".to_string())),
            }),
        ),
    }
}

async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let token = match headers.get("X-Session-Token") {
        Some(val) => match val.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(SuccessResponse { ok: false }),
                );
            }
        },
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(SuccessResponse { ok: false }),
            );
        }
    };

    match auth::delete_session(&state.db, &token).await {
        Ok(_) => {
            tracing::info!("User logged out");
            (StatusCode::OK, Json(SuccessResponse { ok: true }))
        }
        Err(e) => {
            tracing::error!("Logout error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(SuccessResponse { ok: false }),
            )
        }
    }
}

// ── Match History Handlers ────────────────────────────────────────────────

async fn get_matches(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let user = match require_user(&state.db, &headers, None).await {
        Ok(u) => u,
        Err((status, json)) => {
            return (
                status,
                Json(MatchesResponse {
                    ok: false,
                    matches: None,
                    error: Some(json.0.error.unwrap_or_else(|| "Unauthorized".to_string())),
                }),
            );
        }
    };

    let conn = match state.db.conn().await {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("DB connection error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MatchesResponse {
                    ok: false,
                    matches: None,
                    error: Some("Internal server error".to_string()),
                }),
            );
        }
    };

    let mut rows = match conn
        .query(
            "SELECT id, user_id, date, bot_difficulty, trump_suit, bet_size, bet_winner, \
             partner, won_match, player1_sets, player2_sets, player3_sets, player4_sets, \
             player1_hand, player2_hand, player3_hand, player4_hand, sets_data, players \
             FROM matches WHERE user_id = ?1 ORDER BY date DESC",
            libsql::params![user.id],
        )
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("DB query error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MatchesResponse {
                    ok: false,
                    matches: None,
                    error: Some("Internal server error".to_string()),
                }),
            );
        }
    };

    let mut matches = Vec::new();
    loop {
        match rows.next().await {
            Ok(Some(row)) => {
                matches.push(MatchRow {
                    id: row.get::<i64>(0).unwrap_or(0),
                    user_id: row.get::<i64>(1).unwrap_or(0),
                    date: row.get::<i64>(2).unwrap_or(0),
                    bot_difficulty: row.get::<String>(3).unwrap_or_default(),
                    trump_suit: row.get::<String>(4).unwrap_or_default(),
                    bet_size: row.get::<i64>(5).unwrap_or(0),
                    bet_winner: row.get::<i64>(6).unwrap_or(0),
                    partner: row.get::<Option<i64>>(7).unwrap_or(None),
                    won_match: row.get::<Option<i64>>(8).unwrap_or(None),
                    player1_sets: row.get::<i64>(9).unwrap_or(0),
                    player2_sets: row.get::<i64>(10).unwrap_or(0),
                    player3_sets: row.get::<i64>(11).unwrap_or(0),
                    player4_sets: row.get::<i64>(12).unwrap_or(0),
                    player1_hand: row.get::<String>(13).unwrap_or_default(),
                    player2_hand: row.get::<String>(14).unwrap_or_default(),
                    player3_hand: row.get::<String>(15).unwrap_or_default(),
                    player4_hand: row.get::<String>(16).unwrap_or_default(),
                    sets_data: row.get::<Option<String>>(17).unwrap_or(None),
                    players: row.get::<Option<String>>(18).unwrap_or(None),
                });
            }
            Ok(None) => break,
            Err(e) => {
                tracing::error!("DB row read error: {}", e);
                break;
            }
        }
    }

    (
        StatusCode::OK,
        Json(MatchesResponse {
            ok: true,
            matches: Some(matches),
            error: None,
        }),
    )
}

async fn save_match(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<SaveMatchRequest>,
) -> impl IntoResponse {
    let user = match require_user(&state.db, &headers, None).await {
        Ok(u) => u,
        Err((status, json)) => {
            return (
                status,
                Json(AuthResponse {
                    ok: false,
                    token: None,
                    user_id: None,
                    username: None,
                    error: Some(json.0.error.unwrap_or_else(|| "Unauthorized".to_string())),
                }),
            );
        }
    };

    let conn = match state.db.conn().await {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("DB connection error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthResponse {
                    ok: false,
                    token: None,
                    user_id: None,
                    username: None,
                    error: Some("Internal server error".to_string()),
                }),
            );
        }
    };

    let result = conn
        .execute(
            "INSERT INTO matches (user_id, date, bot_difficulty, trump_suit, bet_size, \
             bet_winner, partner, won_match, player1_sets, player2_sets, player3_sets, \
             player4_sets, player1_hand, player2_hand, player3_hand, player4_hand, sets_data, players) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
            libsql::params![
                user.id,
                payload.date,
                payload.bot_difficulty,
                payload.trump_suit,
                payload.bet_size,
                payload.bet_winner,
                payload.partner,
                payload.won_match,
                payload.player1_sets,
                payload.player2_sets,
                payload.player3_sets,
                payload.player4_sets,
                payload.player1_hand,
                payload.player2_hand,
                payload.player3_hand,
                payload.player4_hand,
                payload.sets_data,
                payload.players,
            ],
        )
        .await;

    match result {
        Ok(_) => {
            tracing::info!("Match saved for user_id={}", user.id);
            (
                StatusCode::CREATED,
                Json(AuthResponse {
                    ok: true,
                    token: None,
                    user_id: None,
                    username: None,
                    error: None,
                }),
            )
        }
        Err(e) => {
            tracing::error!("Match insert error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(AuthResponse {
                    ok: false,
                    token: None,
                    user_id: None,
                    username: None,
                    error: Some("Failed to save match".to_string()),
                }),
            )
        }
    }
}

// ── Game Room Handlers ────────────────────────────────────────────────────

async fn create_room(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // Auth check
    let user = match require_user(&state.db, &headers, None).await {
        Ok(u) => u,
        Err((status, json)) => return (
            status,
            Json(CreateRoomResponse {
                room_id: Uuid::nil(),
                player_id: Uuid::nil(),
                seat_index: 0,
            }),
        ),
    };

    let mut room = crate::session::GameRoom::new();
    let room_id = room.room_id;

    // Auto-join the creating player using auth username
    let (player_id, seat_index) = match room.add_player(&user.username) {
        Ok((pid, si)) => (pid, si),
        Err(e) => return (
            StatusCode::BAD_REQUEST,
            Json(CreateRoomResponse {
                room_id,
                player_id: Uuid::nil(),
                seat_index: 0,
            }),
        ),
    };

    let mut rooms = state.rooms.write().await;
    rooms.insert(room_id, room);

    tracing::info!("Created room {} with player '{}' at seat {}", room_id, user.username, seat_index);
    (StatusCode::CREATED, Json(CreateRoomResponse { room_id, player_id, seat_index }))
}

async fn join_room(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(room_id): Path<Uuid>,
) -> impl IntoResponse {
    // Auth check
    let user = match require_user(&state.db, &headers, None).await {
        Ok(u) => u,
        Err((status, json)) => return (
            status,
            Json(JoinRoomResponse {
                player_id: Uuid::nil(),
                seat_index: 0,
                room_id,
            }),
        ),
    };

    let mut rooms = state.rooms.write().await;
    let room = match rooms.get_mut(&room_id) {
        Some(r) => r,
        None => return (StatusCode::NOT_FOUND, Json(JoinRoomResponse {
            player_id: Uuid::nil(),
            seat_index: 0,
            room_id,
        })),
    };

    match room.add_player(&user.username) {
        Ok((player_id, seat_index)) => {
            tracing::info!(
                "Player '{}' joined room {} as seat {}",
                user.username, room_id, seat_index
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
        None => return (StatusCode::NOT_FOUND, Json(NewGameResponse {
            ok: false,
            room_id: None,
            state: None,
            error: Some("Room not found".to_string()),
        })),
    };

    let connected_count = room.sessions.len();
    if connected_count == 0 {
        return (StatusCode::BAD_REQUEST, Json(NewGameResponse {
            ok: false,
            room_id: None,
            state: None,
            error: Some("No players in room".to_string()),
        }));
    }

    // Fill empty seats with bots
    let bot_names = ["Bot-Alpha", "Bot-Beta", "Bot-Gamma"];
    let mut bot_idx = 0;
    for seat in 0..4 {
        let has_player = room.sessions.values().any(|s| s.seat_index == seat);
        if !has_player && bot_idx < bot_names.len() {
            let name = bot_names[bot_idx];
            bot_idx += 1;
            // Set bot name on the table
            room.table.players[seat].name = name.to_string();
            // Add a pseudo-session for the bot (no auth, marked by "Bot-" prefix)
            // The advance endpoint checks for "Bot-" prefix to identify bots
        }
    }

    // Deal cards
    room.table.deal();
    room.is_started = true;

    // Process initial bot turns (bidding starts with P1)
    // P1 (index 0) always starts bidding — if P1 is human, no initial bot actions
    // The advance endpoint handles bot turns after human actions

    let state_resp = build_table_state(&room.table);
    tracing::info!("Game started in room {} ({} human players + {} bots)", room_id, connected_count, 4 - connected_count);
    (StatusCode::OK, Json(NewGameResponse {
        ok: true,
        room_id: Some(room_id),
        state: Some(state_resp),
        error: None,
    }))
}

/// GET /room/{id}/info — lobby info (players, phase, etc.)
async fn get_room_info(
    State(state): State<AppState>,
    Path(room_id): Path<Uuid>,
) -> impl IntoResponse {
    let rooms = state.rooms.read().await;
    let room = match rooms.get(&room_id) {
        Some(r) => r,
        None => return (StatusCode::NOT_FOUND, Json(RoomInfoResponse {
            room_id,
            is_started: false,
            phase: String::new(),
            players: vec![],
        })),
    };

    let players: Vec<RoomPlayerInfo> = room.sessions.values().map(|s| {
        RoomPlayerInfo {
            name: s.player_name.clone(),
            seat_index: s.seat_index,
            is_bot: s.player_name.starts_with("Bot-"),
        }
    }).collect();

    (StatusCode::OK, Json(RoomInfoResponse {
        room_id,
        is_started: room.is_started,
        phase: format!("{:?}", room.table.phase),
        players,
    }))
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
            current_trick_cards: Vec::new(),
            current_trick_start_player: 0,
            previous_trick_cards: Vec::new(),
            previous_trick_winner: None,
            previous_trick_start_player: 0,
            call_history: Vec::new(),
            call_history_start_player: 0,
            partner_card: None,
            trump_played: false,
            lead_suit: None,
            completed_sets: Vec::new(),
        })),
    };

    (
        StatusCode::OK,
        Json(build_table_state(&room.table)),
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

// ── Helper: build TableStateResponse from a Table ─────────────────────────

fn build_table_state(table: &game_core::Table) -> TableStateResponse {
    let hands: Vec<String> = table
        .players
        .iter()
        .map(|p| {
            p.hand
                .iter()
                .map(|c| c.to_ascii_string())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect();

    let phase = format!("{:?}", table.phase);
    let current_player = table.current_player_index();
    let trump_suit = table.trump_suit.map(|s| format!("{:?}", s));
    let sets_won = table.sets_won.to_vec();
    let is_finished = table.phase == game_core::GamePhase::Finished;

    // Current trick: cards in play
    let current_trick_cards = table.current_set_cards.clone();
    let n = current_trick_cards.len();
    // Who led the current trick: if N cards in the trick and current_player
    // is the next player to play, then leader = (current_player - N + 4) % 4
    let current_trick_start_player = if n > 0 {
        (current_player + 4 - n % 4) % 4
    } else {
        current_player
    };

    // Previous trick: last completed set
    let previous_trick_start_player;
    let (previous_trick_cards, previous_trick_winner) =
        if let Some(last_set) = table.completed_sets.last() {
            // Find the leader of the last completed set.
            // First set is led by (bet_winner + 1) % 4.
            // Each subsequent set is led by the previous set's winner.
            previous_trick_start_player = if table.completed_sets.len() > 1 {
                // The winner of the second-to-last set leads the last set
                table.completed_sets[table.completed_sets.len() - 2].winner
            } else {
                // Only one set completed — first set leader
                (table.bet_winner.unwrap_or(0) + 1) % 4
            };
            (last_set.cards.to_vec(), Some(last_set.winner))
        } else {
            previous_trick_start_player = 0;
            (Vec::new(), None)
        };

    // Call history from the auction
    let call_history = table
        .auction
        .as_ref()
        .map(|a| a.call_history.clone())
        .unwrap_or_default();
    // Player 0 (index 0) always starts bidding in our implementation
    let call_history_start_player = 0usize;

    TableStateResponse {
        phase,
        hands,
        current_player,
        bet_size: table.bet_size,
        trump_suit,
        bet_winner: table.bet_winner,
        partner_idx: table.partner_idx,
        sets_won,
        completed_set_count: table.completed_sets.len(),
        is_finished,
        current_trick_cards,
        current_trick_start_player,
        previous_trick_cards,
        previous_trick_winner,
        previous_trick_start_player,
        call_history,
        call_history_start_player,
        partner_card: table.partner_card,
        trump_played: table.trump_played,
        lead_suit: table.lead_suit.map(|s| format!("{:?}", s)),
        completed_sets: table.completed_sets.clone(),
    }
}

// ── Single-player game handlers ───────────────────────────────────────────

async fn create_single_player_game(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<NewGameRequest>,
) -> impl IntoResponse {
    // Auth check
    let user = match require_user(&state.db, &headers, None).await {
        Ok(u) => u,
        Err((status, json)) => {
            return (
                status,
                Json(NewGameResponse {
                    ok: false,
                    room_id: None,
                    state: None,
                    error: Some(json.0.error.unwrap_or_else(|| "Unauthorized".to_string())),
                }),
            );
        }
    };

    // Parse difficulty
    let difficulty = match payload.difficulty.to_lowercase().as_str() {
        "easy" => BotDifficulty::Easy,
        "medium" => BotDifficulty::Medium,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(NewGameResponse {
                    ok: false,
                    room_id: None,
                    state: None,
                    error: Some("Difficulty must be 'Easy' or 'Medium'".to_string()),
                }),
            );
        }
    };

    // Create the single-player game
    let (mut room, session) =
        game_session::new_single_player_game(&user.username, difficulty);

    // Run initial bot turns (if it's not the human's turn to go first)
    game_session::process_bot_turns(&mut room.table, session.human_seat_index, difficulty);

    let state_resp = build_table_state(&room.table);
    let room_id = room.room_id;

    // Store the room in shared state
    let mut rooms = state.rooms.write().await;
    rooms.insert(room_id, room);

    tracing::info!(
        "Single-player game created: room={}, human={} (seat {}), difficulty={:?}",
        room_id,
        user.username,
        session.human_seat_index,
        difficulty,
    );

    (
        StatusCode::CREATED,
        Json(NewGameResponse {
            ok: true,
            room_id: Some(room_id),
            state: Some(state_resp),
            error: None,
        }),
    )
}

async fn single_player_action(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(room_id): Path<Uuid>,
    Json(payload): Json<GameActionRequest>,
) -> impl IntoResponse {
    // Auth check
    let user = match require_user(&state.db, &headers, None).await {
        Ok(u) => u,
        Err((status, json)) => {
            return (
                status,
                Json(GameActionResponse {
                    ok: false,
                    state: None,
                    error: Some(json.0.error.unwrap_or_else(|| "Unauthorized".to_string())),
                }),
            );
        }
    };

    // Find the room
    let mut rooms = state.rooms.write().await;
    let room = match rooms.get_mut(&room_id) {
        Some(r) => r,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(GameActionResponse {
                    ok: false,
                    state: None,
                    error: Some("Room not found".to_string()),
                }),
            );
        }
    };

    // Determine the human seat index and difficulty.
    // For single-player rooms, we need to find the human player.
    // The human is the one whose name matches the authenticated user.
    let human_seat = match room
        .sessions
        .values()
        .find(|s| s.player_name == user.username)
    {
        Some(session) => session.seat_index,
        None => {
            return (
                StatusCode::FORBIDDEN,
                Json(GameActionResponse {
                    ok: false,
                    state: None,
                    error: Some("You are not a player in this room".to_string()),
                }),
            );
        }
    };

    // Build the human action from the request
    let human_action = match payload.action_type.as_str() {
        "bid" => {
            let call = match payload.call {
                Some(c) => c,
                None => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(GameActionResponse {
                            ok: false,
                            state: None,
                            error: Some("Missing 'call' field for bid action".to_string()),
                        }),
                    );
                }
            };
            HumanAction::Call(call)
        }
        "play" => {
            let card = match payload.card {
                Some(c) => c,
                None => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(GameActionResponse {
                            ok: false,
                            state: None,
                            error: Some("Missing 'card' field for play action".to_string()),
                        }),
                    );
                }
            };
            HumanAction::PlayCard(card)
        }
        "selectPartner" => {
            let card = match payload.card {
                Some(c) => c,
                None => {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(GameActionResponse {
                            ok: false,
                            state: None,
                            error: Some(
                                "Missing 'card' field for selectPartner action".to_string(),
                            ),
                        }),
                    );
                }
            };
            HumanAction::SelectPartner(card)
        }
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(GameActionResponse {
                    ok: false,
                    state: None,
                    error: Some(format!(
                        "Unknown action type '{}'. Use 'bid', 'play', or 'selectPartner'",
                        payload.action_type
                    )),
                }),
            );
        }
    };

    // Apply the human move and process bot turns.
    // We determine difficulty by checking whether the room has bot players.
    // Bot players have names starting with "Bot-".
    let has_bots = room.sessions.values().any(|s| s.player_name.starts_with("Bot-"));
    let difficulty = if has_bots {
        BotDifficulty::Easy
    } else {
        // Default difficulty for non-bot rooms; shouldn't happen for single-player
        BotDifficulty::Easy
    };

    match game_session::action_human_move(
        &mut room.table,
        human_seat,
        &human_action,
        difficulty,
    ) {
        Ok(()) => {
            let state_resp = build_table_state(&room.table);
            (
                StatusCode::OK,
                Json(GameActionResponse {
                    ok: true,
                    state: Some(state_resp),
                    error: None,
                }),
            )
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(GameActionResponse {
                ok: false,
                state: None,
                error: Some(e.to_string()),
            }),
        ),
    }
}

// ── Advance bot turn endpoint ─────────────────────────────────────────────

/// POST /api/game/{room_id}/advance
///
/// Advances exactly one bot turn, returning the updated game state.
/// The frontend polls this endpoint at human-configured speed to create
/// a visual delay between bot moves.
async fn advance_game(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(room_id): Path<Uuid>,
) -> impl IntoResponse {
    // Auth check
    let user = match require_user(&state.db, &headers, None).await {
        Ok(u) => u,
        Err((status, json)) => {
            return (
                status,
                Json(GameActionResponse {
                    ok: false,
                    state: None,
                    error: Some(json.0.error.unwrap_or_else(|| "Unauthorized".to_string())),
                }),
            );
        }
    };

    // Find the room
    let mut rooms = state.rooms.write().await;
    let room = match rooms.get_mut(&room_id) {
        Some(r) => r,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(GameActionResponse {
                    ok: false,
                    state: None,
                    error: Some("Room not found".to_string()),
                }),
            );
        }
    };

    // Find the human seat (the authenticated user's seat)
    let human_seat = match room
        .sessions
        .values()
        .find(|s| s.player_name == user.username)
    {
        Some(session) => session.seat_index,
        None => {
            return (
                StatusCode::FORBIDDEN,
                Json(GameActionResponse {
                    ok: false,
                    state: None,
                    error: Some("You are not a player in this room".to_string()),
                }),
            );
        }
    };

    // Determine difficulty (same logic as single_player_action)
    let has_bots = room.sessions.values().any(|s| s.player_name.starts_with("Bot-"));
    let difficulty = if has_bots {
        BotDifficulty::Easy
    } else {
        BotDifficulty::Easy
    };

    // Advance exactly one bot turn
    match game_session::advance_one_turn(&mut room.table, human_seat, difficulty) {
        Ok(_advanced) => {
            let state_resp = build_table_state(&room.table);
            (
                StatusCode::OK,
                Json(GameActionResponse {
                    ok: true,
                    state: Some(state_resp),
                    error: None,
                }),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(GameActionResponse {
                ok: false,
                state: None,
                error: Some(e.to_string()),
            }),
        ),
    }
}

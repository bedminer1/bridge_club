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
use crate::db::{DbPool, MatchResponse, ParticipantResponse, UserRow};
use crate::game_session;
use crate::session::AppState;

// ── Request / Response types ──────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CheckUsernameQuery {
    pub username: String,
}

#[derive(Serialize)]
pub struct CheckUsernameResponse {
    pub ok: bool,
    pub available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Deserialize)]
pub struct ChangeNameRequest {
    pub password: String,
    pub new_username: String,
}

// ── Request / Response types ──────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomInfoResponse {
    pub room_id: Uuid,
    pub is_started: bool,
    pub phase: String,
    pub players: Vec<RoomPlayerInfo>,
    pub hidden_mode: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoomPlayerInfo {
    pub name: String,
    pub seat_index: usize,
    pub is_bot: bool,
}

#[derive(Debug, Serialize)]
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
    pub player_names: Vec<String>,
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
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub id: i64,
    pub username: String,
    pub games_played: i64,
    pub games_won: i64,
    pub total_sets_won: i64,
    pub most_sets_won: i64,
    pub elo: i64,
}

#[derive(Serialize)]
pub struct SuccessResponse {
    pub ok: bool,
}

// ── Match request / response types ─────────────────────────────────────────

/// One participant being saved as part of a match.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveParticipant {
    pub username: String,
    pub seat_index: i64,
    pub team: i64,
    pub sets_won: i64,
    pub cards_played: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveMatchRequest {
    pub room_id: Option<String>,
    pub created_at: i64,
    pub trump_suit: String,
    pub bet_size: i64,
    pub bet_winner_idx: i64,
    pub partner_idx: Option<i64>,
    pub partner_card: Option<String>,
    pub winning_team: i64,
    pub sets_data: Option<String>,
    pub match_type: String,
    pub is_hidden: bool,
    pub participants: Vec<SaveParticipant>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchesResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matches: Option<Vec<MatchResponse>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_more_older: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_more_newer: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oldest_match_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub newest_match_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchHistoryQuery {
    pub limit: Option<usize>,
    pub before_id: Option<i64>,
    pub after_id: Option<i64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardEntry {
    pub id: i64,
    pub username: String,
    pub games_played: i64,
    pub games_won: i64,
    pub winrate: f64,
    pub total_sets_won: i64,
    pub most_sets_won: i64,
    pub elo: i64,
}

#[derive(Serialize)]
pub struct LeaderboardResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entries: Option<Vec<LeaderboardEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Deserialize)]
pub struct SessionQuery {
    pub token: Option<String>,
}

// ── Feedback request / response types ──────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedbackRequest {
    pub match_id: i64,
    pub player_id: i64,
    pub feature_requests: String,
    pub bug_reports: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedbackResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
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
    if let Some(val) = headers.get("X-Session-Token") {
        if let Ok(s) = val.to_str() {
            if !s.is_empty() {
                return Some(s.to_string());
            }
        }
    }
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
        .route("/", get(|| async { "Bridge Club API v0.2 — New match schema" }))
        // Room info & state routes
        .route("/api/rooms/{room_id}/info", get(get_room_info))
        .route("/room/{room_id}/state", get(get_table_state))
        // Auth routes
        .route("/api/auth/signup", post(signup))
        .route("/api/auth/login", post(login))
        .route("/api/auth/session", get(get_session))
        .route("/api/auth/logout", post(logout))
        .route("/api/auth/check-username", get(check_username))
        .route("/api/auth/change-password", post(change_password))
        .route("/api/auth/change-name", post(change_name))
        // Match history routes
        .route("/api/matches", get(get_matches).post(save_match))
        .route("/api/matches/{match_id}", get(get_match))
        // Leaderboard
        .route("/api/leaderboard", get(get_leaderboard))
        // Webhook for auto-deploy
        .route("/api/deploy", post(deploy_webhook))
        // Feedback
        .route("/api/feedback", post(submit_feedback))
        // Single-player game routes
        .route("/api/game/new", post(create_single_player_game))
        .with_state(state)
}

// ── Auth Handlers ─────────────────────────────────────────────────────────

async fn signup(
    State(state): State<AppState>,
    Json(payload): Json<SignupRequest>,
) -> impl IntoResponse {
    if payload.username.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(AuthResponse {
                ok: false,
                token: None,
                user_id: None,
                username: None,
                error: Some("Username is required".to_string()),
            }),
        );
    }

    let conn = state.db.conn().await;

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

    let insert = conn
        .execute(
            "INSERT INTO users (username, password) VALUES (?1, ?2)",
            libsql::params![payload.username.clone(), password_hash],
        )
        .await;

    let user_id = match insert {
        Ok(_) => {
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

    drop(conn);

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
    if payload.username.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(AuthResponse {
                ok: false,
                token: None,
                user_id: None,
                username: None,
                error: Some("Username is required".to_string()),
            }),
        );
    }

    let conn = state.db.conn().await;

    let mut rows = match conn
        .query(
            "SELECT id, username, password, games_played, games_won, total_sets_won, most_sets_won, elo FROM users WHERE username = ?1",
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
            // Guest accounts have empty password; any other account requires matching hash
            if !db_password.is_empty() {
                let password_hash = auth::hash_password(&payload.password);
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
            }
            // Guest (empty password) or password matches
            UserRow {
                id: row.get::<i64>(0).unwrap(),
                username: row.get::<String>(1).unwrap(),
                password: db_password,
                games_played: row.get::<i64>(3).unwrap_or(0),
                games_won: row.get::<i64>(4).unwrap_or(0),
                total_sets_won: row.get::<i64>(5).unwrap_or(0),
                most_sets_won: row.get::<i64>(6).unwrap_or(0),
                elo: row.get::<i64>(7).unwrap_or(500),
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

    drop(conn);

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
                    error: Some("Internal server error".to_string()),
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
                    games_played: user.games_played,
                    games_won: user.games_won,
                    total_sets_won: user.total_sets_won,
                    most_sets_won: user.most_sets_won,
                    elo: user.elo,
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

// ── Account Management Handlers ──────────────────────────────────────────

/// GET /api/auth/check-username?username=X — check if a username is available.
async fn check_username(
    Query(query): Query<CheckUsernameQuery>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if query.username.is_empty() || query.username.len() < 2 || query.username.len() > 20 {
        return Json(CheckUsernameResponse {
            ok: false,
            available: false,
            error: Some("Username must be 2-20 characters".to_string()),
        });
    }

    let conn = state.db.conn().await;
    let exists = match conn
        .query("SELECT id FROM users WHERE username = ?1", libsql::params![query.username])
        .await
    {
        Ok(mut rows) => rows.next().await.ok().flatten().is_some(),
        Err(_) => false,
    };

    Json(CheckUsernameResponse {
        ok: true,
        available: !exists,
        error: None,
    })
}

#[derive(Serialize)]
struct ChangeResponse {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// POST /api/auth/change-password — change password for the authenticated user.
async fn change_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ChangePasswordRequest>,
) -> impl IntoResponse {
    let user = match require_user(&state.db, &headers, None).await {
        Ok(u) => u,
        Err((status, json)) => {
            return (status, Json(ChangeResponse {
                ok: false,
                error: Some(json.0.error.unwrap_or_else(|| "Unauthorized".to_string())),
            }));
        }
    };

    if payload.new_password.len() < 6 {
        return (StatusCode::BAD_REQUEST, Json(ChangeResponse {
            ok: false,
            error: Some("New password must be at least 6 characters".to_string()),
        }));
    }

    let current_hash = auth::hash_password(&payload.current_password);
    if current_hash != user.password {
        return (StatusCode::UNAUTHORIZED, Json(ChangeResponse {
            ok: false,
            error: Some("Current password is incorrect".to_string()),
        }));
    }

    let new_hash = auth::hash_password(&payload.new_password);
    let conn = state.db.conn().await;
    let _ = conn.execute(
        "UPDATE users SET password = ?1 WHERE id = ?2",
        libsql::params![new_hash, user.id],
    ).await;

    (StatusCode::OK, Json(ChangeResponse { ok: true, error: None }))
}

/// POST /api/auth/change-name — change username for the authenticated user.
async fn change_name(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ChangeNameRequest>,
) -> impl IntoResponse {
    let user = match require_user(&state.db, &headers, None).await {
        Ok(u) => u,
        Err((status, json)) => {
            return (status, Json(ChangeResponse {
                ok: false,
                error: Some(json.0.error.unwrap_or_else(|| "Unauthorized".to_string())),
            }));
        }
    };

    if payload.new_username.len() < 2 || payload.new_username.len() > 20 {
        return (StatusCode::BAD_REQUEST, Json(ChangeResponse {
            ok: false,
            error: Some("Username must be 2-20 characters".to_string()),
        }));
    }

    let password_hash = auth::hash_password(&payload.password);
    if password_hash != user.password {
        return (StatusCode::UNAUTHORIZED, Json(ChangeResponse {
            ok: false,
            error: Some("Password is incorrect".to_string()),
        }));
    }

    let conn = state.db.conn().await;
    // Check if new username is taken
    if let Ok(mut rows) = conn
        .query("SELECT id FROM users WHERE username = ?1 AND id != ?2", libsql::params![payload.new_username.clone(), user.id])
        .await
    {
        if let Ok(Some(_)) = rows.next().await {
            return (StatusCode::CONFLICT, Json(ChangeResponse {
                ok: false,
                error: Some("Username already taken".to_string()),
            }));
        }
    }

    let _ = conn.execute(
        "UPDATE users SET username = ?1 WHERE id = ?2",
        libsql::params![payload.new_username, user.id],
    ).await;

    (StatusCode::OK, Json(ChangeResponse { ok: true, error: None }))
}

// ── Match History Handlers ────────────────────────────────────────────────

/// Build a MatchResponse by querying match + participants for a given match_id.
async fn build_match_response(conn: &libsql::Connection, match_id: i64) -> Option<MatchResponse> {
    let mut mrows = conn
        .query(
            "SELECT id, room_id, created_at, trump_suit, bet_size, bet_winner_idx, partner_idx, \
             partner_card, winning_team, team1_sets, team2_sets, sets_data, match_type, is_hidden \
             FROM matches WHERE id = ?1",
            libsql::params![match_id],
        )
        .await
        .ok()?;

    let mrow = mrows.next().await.ok()??;

    let mut participants = Vec::new();
    let mut prows = conn
        .query(
            "SELECT id, user_id, seat_index, team, sets_won, cards_played, hand_preview, elo_change \
             FROM match_participants WHERE match_id = ?1 ORDER BY seat_index",
            libsql::params![match_id],
        )
        .await
        .ok()?;

    loop {
        match prows.next().await {
            Ok(Some(row)) => {
                participants.push(ParticipantResponse {
                    id: row.get::<i64>(0).unwrap_or(0),
                    user_id: row.get::<i64>(1).unwrap_or(0),
                    seat_index: row.get::<i64>(2).unwrap_or(0),
                    team: row.get::<i64>(3).unwrap_or(1),
                    sets_won: row.get::<i64>(4).unwrap_or(0),
                    cards_played: row.get::<String>(5).unwrap_or_default(),
                    hand_preview: row.get::<Option<String>>(6).unwrap_or(None),
                    elo_change: row.get::<i64>(7).unwrap_or(0),
                });
            }
            Ok(None) => break,
            Err(_) => break,
        }
    }

    Some(MatchResponse {
        id: mrow.get::<i64>(0).unwrap_or(0),
        room_id: mrow.get::<Option<String>>(1).unwrap_or(None),
        created_at: mrow.get::<i64>(2).unwrap_or(0),
        trump_suit: mrow.get::<String>(3).unwrap_or_default(),
        bet_size: mrow.get::<i64>(4).unwrap_or(0),
        bet_winner_idx: mrow.get::<i64>(5).unwrap_or(0),
        partner_idx: mrow.get::<Option<i64>>(6).unwrap_or(None),
        partner_card: mrow.get::<Option<String>>(7).unwrap_or(None),
        winning_team: mrow.get::<i64>(8).unwrap_or(1),
        team1_sets: mrow.get::<i64>(9).unwrap_or(0),
        team2_sets: mrow.get::<i64>(10).unwrap_or(0),
        sets_data: mrow.get::<Option<String>>(11).unwrap_or(None),
        match_type: mrow.get::<String>(12).unwrap_or_default(),
        is_hidden: mrow.get::<bool>(13).unwrap_or(true),
        participants,
    })
}

/// GET /api/matches — list matches for the authenticated user.
async fn get_matches(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<MatchHistoryQuery>,
) -> impl IntoResponse {
    let user = match require_user(&state.db, &headers, None).await {
        Ok(u) => u,
        Err((status, json)) => {
            return (
                status,
                Json(MatchesResponse {
                    ok: false,
                    matches: None,
                    has_more_older: None,
                    has_more_newer: None,
                    oldest_match_id: None,
                    newest_match_id: None,
                    error: Some(json.0.error.unwrap_or_else(|| "Unauthorized".to_string())),
                }),
            );
        }
    };

    if query.before_id.is_some() && query.after_id.is_some() {
        return (
            StatusCode::BAD_REQUEST,
            Json(MatchesResponse {
                ok: false,
                matches: None,
                has_more_older: None,
                has_more_newer: None,
                oldest_match_id: None,
                newest_match_id: None,
                error: Some("Use either beforeId or afterId, not both".to_string()),
            }),
        );
    }

    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let fetch_limit = limit + 1;

    let conn = state.db.conn().await;

    // Get match IDs for this user via match_participants, with optional cursors.
    let mut id_rows = match if let Some(before_id) = query.before_id {
        conn.query(
            "SELECT DISTINCT match_id FROM match_participants WHERE user_id = ?1 AND match_id < ?2 ORDER BY match_id DESC LIMIT ?3",
            libsql::params![user.id, before_id, fetch_limit as i64],
        ).await
    } else if let Some(after_id) = query.after_id {
        conn.query(
            "SELECT DISTINCT match_id FROM match_participants WHERE user_id = ?1 AND match_id > ?2 ORDER BY match_id ASC LIMIT ?3",
            libsql::params![user.id, after_id, fetch_limit as i64],
        ).await
    } else {
        conn.query(
            "SELECT DISTINCT match_id FROM match_participants WHERE user_id = ?1 ORDER BY match_id DESC LIMIT ?2",
            libsql::params![user.id, fetch_limit as i64],
        ).await
    } {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("DB query error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(MatchesResponse {
                    ok: false,
                    matches: None,
                    has_more_older: None,
                    has_more_newer: None,
                    oldest_match_id: None,
                    newest_match_id: None,
                    error: Some("Internal server error".to_string()),
                }),
            );
        }
    };

    let mut match_ids = Vec::new();
    loop {
        match id_rows.next().await {
            Ok(Some(row)) => match_ids.push(row.get::<i64>(0).unwrap_or(0)),
            Ok(None) => break,
            Err(_) => break,
        }
    }

    let has_more = match_ids.len() > limit;
    if has_more {
        match_ids.truncate(limit);
    }

    let mut matches = Vec::new();
    for mid in match_ids {
        if let Some(m) = build_match_response(&conn, mid).await {
            matches.push(m);
        }
    }

    matches.sort_by(|a, b| b.id.cmp(&a.id));

    let newest_match_id = matches.first().map(|m| m.id);
    let oldest_match_id = matches.last().map(|m| m.id);

    (
        StatusCode::OK,
        Json(MatchesResponse {
            ok: true,
            matches: Some(matches),
            has_more_older: if query.after_id.is_some() { None } else { Some(has_more) },
            has_more_newer: if query.after_id.is_some() { Some(has_more) } else { None },
            oldest_match_id,
            newest_match_id,
            error: None,
        }),
    )
}

/// GET /api/matches/{match_id} — get a single match by ID.
async fn get_match(
    State(state): State<AppState>,
    Path(match_id): Path<i64>,
) -> impl IntoResponse {
    let conn = state.db.conn().await;

    match build_match_response(&conn, match_id).await {
        Some(m) => (
            StatusCode::OK,
            Json(serde_json::json!({ "ok": true, "match": m })),
        ),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "ok": false, "error": "Match not found" })),
        ),
    }
}

/// POST /api/matches — save a completed match.
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
                Json(serde_json::json!({
                    "ok": false,
                    "error": json.0.error.clone().unwrap_or_else(|| "Unauthorized".to_string()),
                })),
            );
        }
    };

    let conn = state.db.conn().await;

    // Dedup: if a match with this room_id already exists, skip save
    if let Some(ref room_id) = payload.room_id {
        if let Ok(mut rows) = conn
            .query(
                "SELECT id FROM matches WHERE room_id = ?1 LIMIT 1",
                libsql::params![room_id.clone()],
            )
            .await
        {
            if let Ok(Some(_)) = rows.next().await {
                tracing::info!("Match for room {} already saved, skipping duplicate", room_id);
                return (
                    StatusCode::OK,
                    Json(serde_json::json!({ "ok": true })),
                );
            }
        }
    }

    // Resolve usernames to user_ids
    let mut resolved_participants: Vec<(i64, i64, i64, i64, String, String)> = Vec::new();
    let mut player_ids: [i64; 4] = [0; 4];

    for p in &payload.participants {
        let uid = if p.username.starts_with("Bot-") {
            match p.username.as_str() {
                "Bot-Alpha" => 1i64,
                "Bot-Beta" => 2i64,
                "Bot-Gamma" => 3i64,
                _ => 0i64,
            }
        } else if p.username == user.username {
            user.id
        } else {
            // Look up other human players
            let mut found = 0i64;
            if let Ok(mut rows) = conn
                .query("SELECT id FROM users WHERE username = ?1", libsql::params![p.username.clone()])
                .await
            {
                if let Ok(Some(row)) = rows.next().await {
                    found = row.get::<i64>(0).unwrap_or(0);
                }
            }
            found
        };

        let seat = p.seat_index as usize;
        if seat < 4 {
            player_ids[seat] = uid;
        }

        // Generate hand preview from cards_played
        let preview = compact_hand_preview(&p.cards_played);

        resolved_participants.push((
            uid,
            p.seat_index,
            p.team,
            p.sets_won,
            p.cards_played.clone(),
            preview,
        ));
    }

    // Compute team total sets
    let team1_sets: i64 = payload.participants.iter()
        .filter(|p| p.team == 1)
        .map(|p| p.sets_won)
        .sum();
    let team2_sets: i64 = payload.participants.iter()
        .filter(|p| p.team == 2)
        .map(|p| p.sets_won)
        .sum();

    // Insert match
    let result = conn
        .execute(
            "INSERT INTO matches (room_id, created_at, trump_suit, bet_size, bet_winner_idx, \
             partner_idx, partner_card, winning_team, team1_sets, team2_sets, sets_data, match_type, is_hidden) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            libsql::params![
                payload.room_id,
                payload.created_at,
                payload.trump_suit,
                payload.bet_size,
                payload.bet_winner_idx,
                payload.partner_idx,
                payload.partner_card,
                payload.winning_team,
                team1_sets,
                team2_sets,
                payload.sets_data,
                payload.match_type,
                payload.is_hidden as i64,
            ],
        )
        .await;

    match result {
        Ok(_) => {
            let inserted_id = conn.last_insert_rowid();
            tracing::info!("Match saved: id={}", inserted_id);

            // Insert participants
            for (uid, seat_idx, team, sets_won, cards_played, preview) in &resolved_participants {
                let _ = conn.execute(
                    "INSERT INTO match_participants (match_id, user_id, seat_index, team, sets_won, cards_played, hand_preview, elo_change) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0)",
                    libsql::params![
                        inserted_id,
                        uid,
                        seat_idx,
                        team,
                        sets_won,
                        cards_played.clone(),
                        preview.clone(),
                    ],
                ).await;
            }

            // Elo computation + stats (only for hidden/rated matches)
            let is_hidden = payload.is_hidden;
            let mut elo_delta_for_user: i64 = 0;

            if is_hidden {
                // Compute Elo
                let k: f64 = 32.0;
                let bet_seat = payload.bet_winner_idx as usize;
                let partner_seat = payload.partner_idx.map(|p| p as usize).unwrap_or(99);

                let team1_seats = [bet_seat, partner_seat];
                let team2_seats: Vec<usize> = (0..4)
                    .filter(|s| *s != bet_seat && *s != partner_seat)
                    .collect();

                // Fetch current Elo for all participants
                let mut elos: [f64; 4] = [500.0; 4];
                for (seat, &pid) in player_ids.iter().enumerate() {
                    if pid > 0 {
                        if let Ok(mut erows) = conn
                            .query("SELECT elo FROM users WHERE id = ?1", libsql::params![pid])
                            .await
                        {
                            if let Ok(Some(erow)) = erows.next().await {
                                elos[seat] = erow.get::<i64>(0).unwrap_or(500) as f64;
                            }
                        }
                    }
                }

                // Team average Elo
                let team1_avg = if !team1_seats.is_empty() {
                    team1_seats.iter().map(|&s| elos[s]).sum::<f64>() / team1_seats.len() as f64
                } else {
                    500.0
                };
                let team2_avg = if !team2_seats.is_empty() {
                    team2_seats.iter().map(|&s| elos[s]).sum::<f64>() / team2_seats.len() as f64
                } else {
                    500.0
                };

                let expected_team1 = 1.0 / (1.0 + 10.0_f64.powf((team2_avg - team1_avg) / 400.0));
                let expected_team2 = 1.0 - expected_team1;

                let team1_won = payload.winning_team == 1;
                let delta1 = k * (if team1_won { 1.0 } else { 0.0 } - expected_team1);
                let delta2 = k * (if team1_won { 0.0 } else { 1.0 } - expected_team2);

                tracing::info!(
                    "Elo: team1_avg={:.1}, team2_avg={:.1}, delta1={:.1}, delta2={:.1}, team1_won={}",
                    team1_avg, team2_avg, delta1, delta2, team1_won
                );

                // Update Elo for all participants
                for (seat, &pid) in player_ids.iter().enumerate() {
                    if pid > 0 {
                        let delta = if seat == bet_seat || seat == partner_seat { delta1 } else { delta2 };
                        let delta_int = delta.round() as i64;
                        if pid == user.id {
                            elo_delta_for_user = delta_int;
                        }
                        let _ = conn.execute(
                            "UPDATE users SET elo = MAX(1, elo + ?1) WHERE id = ?2",
                            libsql::params![delta_int, pid],
                        ).await;
                    }
                }

                // Update stats for ALL participants (humans and bots)
                for (uid, _seat_idx, team, sets_won, _cards_played, _preview) in &resolved_participants {
                    let pid = *uid;
                    if pid == 0 { continue; }
                    let won = *team == payload.winning_team;
                    let _ = conn.execute(
                        "UPDATE users SET \
                         games_played = games_played + 1, \
                         games_won = games_won + ?1, \
                         total_sets_won = total_sets_won + ?2, \
                         most_sets_won = MAX(most_sets_won, ?3) \
                         WHERE id = ?4",
                        libsql::params![
                            if won { 1i64 } else { 0i64 },
                            sets_won,
                            sets_won,
                            pid,
                        ],
                    ).await;
                }

                // Update elo_change on participant records
                for (seat, &pid) in player_ids.iter().enumerate() {
                    if pid > 0 {
                        let delta = if seat == bet_seat || seat == partner_seat { delta1 } else { delta2 };
                        let delta_int = delta.round() as i64;
                        let _ = conn.execute(
                            "UPDATE match_participants SET elo_change = ?1 WHERE match_id = ?2 AND user_id = ?3",
                            libsql::params![delta_int, inserted_id, pid],
                        ).await;
                    }
                }
            }

            // Insert remaining bot slots if any are missing (some seats may not have been sent)
            // But the frontend sends all 4 participants, so this shouldn't be needed.

            (
                StatusCode::CREATED,
                Json(serde_json::json!({
                    "ok": true,
                    "id": inserted_id,
                    "eloChange": elo_delta_for_user,
                })),
            )
        }
        Err(e) => {
            tracing::error!("Match insert error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "ok": false,
                    "error": "Failed to save match",
                })),
            )
        }
    }
}

/// Generate a compact preview string from a JSON array of played cards (frontend PascalCase format).
fn compact_hand_preview(hand_json: &str) -> String {
    let cards: Vec<serde_json::Value> = serde_json::from_str(hand_json).unwrap_or_default();
    let mut out = String::new();
    let rank_map: [&str; 15] = ["", "", "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K", "A"];
    for card in &cards {
        let suit = card["Suit"].as_str().unwrap_or("");
        let suit_letter = match suit {
            "Club" => 'c', "Diamond" => 'd', "Heart" => 'h', "Spades" => 's',
            _ => '?',
        };
        let val = card["Value"].as_i64().unwrap_or(2) as usize;
        let rank_str = rank_map.get(val).unwrap_or(&"?");
        let won = card["WonSet"].as_bool().unwrap_or(false);
        out.push_str(rank_str);
        out.push(suit_letter);
        out.push(if won { 'w' } else { 'l' });
    }
    out
}

// ── Leaderboard ──────────────────────────────────────────────────────────

async fn get_leaderboard(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let conn = state.db.conn().await;

    let mut rows = match conn
        .query(
            "SELECT id, username, games_played, games_won, total_sets_won, most_sets_won, elo
             FROM users WHERE id NOT IN (1, 2, 3) ORDER BY elo DESC, games_played DESC",
            libsql::params![],
        )
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Leaderboard query error: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(LeaderboardResponse {
                    ok: false,
                    entries: None,
                    error: Some("Internal server error".to_string()),
                }),
            );
        }
    };

    let mut entries = Vec::new();
    loop {
        match rows.next().await {
            Ok(Some(row)) => {
                let id: i64 = row.get::<i64>(0).unwrap_or(0);
                let username: String = row.get::<String>(1).unwrap_or_default();
                let games_played: i64 = row.get::<i64>(2).unwrap_or(0);
                let games_won: i64 = row.get::<i64>(3).unwrap_or(0);
                let total_sets_won: i64 = row.get::<i64>(4).unwrap_or(0);
                let most_sets_won: i64 = row.get::<i64>(5).unwrap_or(0);
                let elo: i64 = row.get::<i64>(6).unwrap_or(500);
                let winrate = if games_played > 0 {
                    (games_won as f64) / (games_played as f64)
                } else {
                    0.0
                };
                entries.push(LeaderboardEntry {
                    id,
                    username,
                    games_played,
                    games_won,
                    winrate,
                    total_sets_won,
                    most_sets_won,
                    elo,
                });
            }
            Ok(None) => break,
            Err(e) => {
                tracing::error!("Leaderboard row read error: {}", e);
                break;
            }
        }
    }

    (StatusCode::OK, Json(LeaderboardResponse { ok: true, entries: Some(entries), error: None }))
}

// ── Game Room Handlers ────────────────────────────────────────────────────

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
            hidden_mode: false,
        })),
    };

    let mut players: Vec<RoomPlayerInfo> = room.sessions.values().map(|s| {
        RoomPlayerInfo {
            name: s.player_name.clone(),
            seat_index: s.seat_index,
            is_bot: s.player_name.starts_with("Bot-"),
        }
    }).collect();
    players.sort_by_key(|p| p.seat_index);

    (StatusCode::OK, Json(RoomInfoResponse {
        room_id,
        is_started: room.is_started,
        phase: format!("{:?}", room.table.phase),
        players,
        hidden_mode: room.hidden_mode,
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
            player_names: Vec::new(),
        })),
    };

    (
        StatusCode::OK,
        Json(build_table_state(&room.table)),
    )
}

// ── Helper: build TableStateResponse from a Table ─────────────────────────

pub fn build_table_state(table: &game_core::Table) -> TableStateResponse {
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

    let current_trick_cards = table.current_set_cards.clone();
    let n = current_trick_cards.len();
    let current_trick_start_player = if n > 0 {
        (current_player + 4 - n % 4) % 4
    } else {
        current_player
    };

    let previous_trick_start_player;
    let (previous_trick_cards, previous_trick_winner) =
        if let Some(last_set) = table.completed_sets.last() {
            previous_trick_start_player = if table.completed_sets.len() > 1 {
                table.completed_sets[table.completed_sets.len() - 2].winner
            } else {
                (table.bet_winner.unwrap_or(0) + 1) % 4
            };
            (last_set.cards.to_vec(), Some(last_set.winner))
        } else {
            previous_trick_start_player = 0;
            (Vec::new(), None)
        };

    let call_history = table
        .auction
        .as_ref()
        .map(|a| a.call_history.clone())
        .unwrap_or_default();
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
        player_names: table.players.iter().map(|p| p.name.clone()).collect(),
    }
}

// ── Single-player game handlers ───────────────────────────────────────────

async fn create_single_player_game(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<NewGameRequest>,
) -> impl IntoResponse {
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

    let (mut room, session) =
        game_session::new_single_player_game(&user.username, difficulty);

    game_session::process_bot_turns(&mut room.table, session.human_seat_index, difficulty);

    let state_resp = build_table_state(&room.table);
    let room_id = room.room_id;

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

// ── Deploy Webhook ─────────────────────────────────────────────────────────

use std::process::Command;

async fn deploy_webhook() -> impl IntoResponse {
    tokio::task::spawn_blocking(|| {
        let _ = Command::new("/root/deploy-bridge-club.sh")
            .output()
            .map(|o| {
                let log = format!("STDOUT:\n{}\nSTDERR:\n{}",
                    String::from_utf8_lossy(&o.stdout),
                    String::from_utf8_lossy(&o.stderr));
                let _ = std::fs::write("/tmp/deploy.log", &log);
            });
    });
    (StatusCode::ACCEPTED, Json(serde_json::json!({"ok": true, "message": "Deploy started"})))
}

// ── Feedback ────────────────────────────────────────────────────────────────

async fn submit_feedback(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<FeedbackRequest>,
) -> impl IntoResponse {
    let _user = match require_user(&state.db, &headers, None).await {
        Ok(u) => u,
        Err((status, json)) => {
            return (
                status,
                Json(FeedbackResponse {
                    ok: false,
                    id: None,
                    error: Some(json.0.error.unwrap_or_else(|| "Unauthorized".to_string())),
                }),
            );
        }
    };

    let conn = state.db.conn().await;

    let result = conn
        .execute(
            "INSERT INTO feedback (match_id, player_id, feature_requests, bug_reports) \
             VALUES (?1, ?2, ?3, ?4)",
            libsql::params![
                payload.match_id,
                payload.player_id,
                payload.feature_requests,
                payload.bug_reports,
            ],
        )
        .await;

    match result {
        Ok(_) => {
            let inserted_id = conn.last_insert_rowid();
            tracing::info!("Feedback saved: id={}, match={}, player={}", inserted_id, payload.match_id, payload.player_id);
            (
                StatusCode::CREATED,
                Json(FeedbackResponse {
                    ok: true,
                    id: Some(inserted_id),
                    error: None,
                }),
            )
        }
        Err(e) => {
            tracing::error!("Feedback insert error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(FeedbackResponse {
                    ok: false,
                    id: None,
                    error: Some("Failed to save feedback".to_string()),
                }),
            )
        }
    }
}

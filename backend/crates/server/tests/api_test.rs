//! Integration tests for the Bridge Club API.
//!
//! These tests spin up the full axum application with a temporary SQLite
//! database and test endpoints via `tower::ServiceExt::oneshot`.

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceExt;

use bridge_server::db::{run_migrations, DbPool};
use bridge_server::routes;
use bridge_server::session::{AppState, GameRoom};

/// Build a test app with a fresh SQLite database in a temp directory.
async fn build_test_app() -> (Router, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("Failed to create temp dir");
    let db_path = dir.path().join("test.db");
    let db_path_str = db_path.to_str().unwrap().to_string();

    let pool = bridge_server::db::new_temp(&db_path_str).await.expect("Failed to create test DB pool");
    run_migrations(&pool).await.expect("Failed to run migrations");

    let state = AppState {
        rooms: Arc::new(RwLock::new(std::collections::HashMap::<
            uuid::Uuid,
            GameRoom,
        >::new())),
        db: pool,
    };

    let app = routes::routes(state);
    (app, dir)
}

/// Helper: make a raw request and return the response.
async fn request(app: &Router, req: Request<Body>) -> (StatusCode, Value) {
    // Clone the router since oneshot consumes it
    let mut cloned = app.clone();
    let resp = cloned.oneshot(req).await.unwrap();
    let status = resp.status();
    let body_bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body_bytes).unwrap_or(Value::Null);
    (status, json)
}

/// Helper: sign up a user and return the auth token.
async fn signup(app: &Router, username: &str, password: &str) -> String {
    let body = serde_json::json!({
        "username": username,
        "password": password,
    });
    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/signup")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();
    let (status, json) = request(app, req).await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "Signup failed for {}: {:?}",
        username,
        json
    );
    json["token"].as_str().unwrap().to_string()
}

/// Helper: make an authenticated GET request.
async fn get(app: &Router, uri: &str, token: &str) -> (StatusCode, Value) {
    let req = Request::builder()
        .method("GET")
        .uri(uri)
        .header("x-session-token", token)
        .body(Body::empty())
        .unwrap();
    request(app, req).await
}

/// Helper: make an authenticated POST request.
async fn post(app: &Router, uri: &str, token: &str, body_val: &Value) -> (StatusCode, Value) {
    let body_str = serde_json::to_string(body_val).unwrap();
    let req = Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .header("x-session-token", token)
        .body(Body::from(body_str))
        .unwrap();
    request(app, req).await
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_health_check() {
    let (app, _dir) = build_test_app().await;
    let req = Request::builder()
        .uri("/")
        .body(Body::empty())
        .unwrap();
    let (status, _) = request(&app, req).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_signup_and_login() {
    let (app, _dir) = build_test_app().await;

    // Sign up
    let token = signup(&app, "alice", "secret123").await;
    assert!(!token.is_empty());

    // Bad token should be rejected
    let (status, _) = get(&app, "/api/auth/session?token=bad_token", "").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // Valid session
    let (status, json) = get(&app, &format!("/api/auth/session?token={}", token), "").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["ok"], true);
    assert_eq!(json["user"]["username"], "alice");
    assert!(json["user"]["id"].as_i64().unwrap() > 0);
    assert_eq!(json["user"]["elo"], 500);
}

#[tokio::test]
async fn test_signup_duplicate() {
    let (app, _dir) = build_test_app().await;
    signup(&app, "bob", "pass1").await;

    let body = serde_json::json!({"username": "bob", "password": "pass2"});
    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/signup")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();
    let (status, _) = request(&app, req).await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_login_wrong_password() {
    let (app, _dir) = build_test_app().await;
    signup(&app, "carol", "correct").await;

    let body = serde_json::json!({"username": "carol", "password": "wrong"});
    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();
    let (status, _) = request(&app, req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_leaderboard_without_bots() {
    let (app, _dir) = build_test_app().await;

    let req = Request::builder()
        .uri("/api/leaderboard")
        .body(Body::empty())
        .unwrap();
    let (status, json) = request(&app, req).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["ok"], true);

    let entries = json["entries"].as_array().unwrap();
    // Should NOT include bots (ids 1,2,3)
    let usernames: Vec<&str> = entries
        .iter()
        .map(|e| e["username"].as_str().unwrap())
        .collect();
    assert!(!usernames.contains(&"Bot-Alpha"), "Bots should be hidden from leaderboard");
    assert!(!usernames.contains(&"Bot-Beta"));
    assert!(!usernames.contains(&"Bot-Gamma"));
}

#[tokio::test]
async fn test_leaderboard_with_user() {
    let (app, _dir) = build_test_app().await;
    signup(&app, "dave", "pass").await;

    let req = Request::builder()
        .uri("/api/leaderboard")
        .body(Body::empty())
        .unwrap();
    let (_, json) = request(&app, req).await;
    let entries = json["entries"].as_array().unwrap();
    let dave = entries.iter().find(|e| e["username"] == "dave").unwrap();
    assert_eq!(dave["gamesPlayed"], 0);
    assert_eq!(dave["gamesWon"], 0);
    assert_eq!(dave["elo"], 500);
    assert!(dave["elo"].as_i64().unwrap() >= 500);
}

#[tokio::test]
async fn test_matches_empty() {
    let (app, _dir) = build_test_app().await;
    let token = signup(&app, "eve", "pass").await;

    let (status, json) = get(&app, "/api/matches", &token).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["ok"], true);
    assert_eq!(json["matches"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_matches_requires_auth() {
    let (app, _dir) = build_test_app().await;

    let req = Request::builder()
        .uri("/api/matches")
        .body(Body::empty())
        .unwrap();
    let (status, _) = request(&app, req).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_save_and_retrieve_match() {
    let (app, _dir) = build_test_app().await;
    let token = signup(&app, "frank", "pass").await;

    let match_body = serde_json::json!({
        "date": 1700000000000i64,
        "botDifficulty": "Medium",
        "trumpSuit": "Spades",
        "betSize": 3,
        "betWinner": 0,
        "partner": 2,
        "wonMatch": 1,
        "betWinnerUserId": 0,
        "partnerUserId": 0,
        "winningTeam": 1,
        "player1Sets": 6,
        "player2Sets": 2,
        "player3Sets": 3,
        "player4Sets": 1,
        "player1Hand": "[]",
        "player2Hand": "[]",
        "player3Hand": "[]",
        "player4Hand": "[]",
        "setsData": "[]",
        "players": serde_json::json!([
            {"id": 0, "username": "frank"},
            {"id": 0, "username": "Bot-Alpha"},
            {"id": 0, "username": "Bot-Beta"},
            {"id": 0, "username": "Bot-Gamma"}
        ]).to_string(),
        "roomId": "test-room-1"
    });

    let (status, json) = post(&app, "/api/matches", &token, &match_body).await;
    assert_eq!(status, StatusCode::CREATED, "Save match failed: {:?}", json);
    assert_eq!(json["ok"], true);

    // Retrieve matches
    let (status, json) = get(&app, "/api/matches", &token).await;
    assert_eq!(status, StatusCode::OK);
    let matches = json["matches"].as_array().unwrap();
    assert_eq!(matches.len(), 1);

    let m = &matches[0];
    assert_eq!(m["trumpSuit"], "Spades");
    assert_eq!(m["betSize"], 3);
    assert_eq!(m["botDifficulty"], "Medium");
    assert!(m.get("betWinnerUserId").is_some());
    assert!(m.get("partnerUserId").is_some());
    assert!(m.get("winningTeam").is_some());
    assert!(m.get("eloChange").is_some());
}

#[tokio::test]
async fn test_save_match_dedup_by_room() {
    let (app, _dir) = build_test_app().await;
    let token = signup(&app, "grace", "pass").await;

    let match_body = serde_json::json!({
        "date": 1700000000000i64,
        "botDifficulty": "Easy",
        "trumpSuit": "Hearts",
        "betSize": 2,
        "betWinner": 1,
        "partner": 3,
        "wonMatch": 0,
        "betWinnerUserId": 0,
        "partnerUserId": 0,
        "winningTeam": 2,
        "player1Sets": 2,
        "player2Sets": 4,
        "player3Sets": 3,
        "player4Sets": 3,
        "player1Hand": "[]",
        "player2Hand": "[]",
        "player3Hand": "[]",
        "player4Hand": "[]",
        "setsData": "[]",
        "players": serde_json::json!([
            {"id": 0, "username": "grace"},
            {"id": 0, "username": "Bot-Alpha"},
            {"id": 0, "username": "Bot-Beta"},
            {"id": 0, "username": "Bot-Gamma"}
        ]).to_string(),
        "roomId": "dedup-room"
    });

    // First save
    let (status, _) = post(&app, "/api/matches", &token, &match_body).await;
    assert_eq!(status, StatusCode::CREATED);

    // Second save with same room_id should dedup
    let (status, _) = post(&app, "/api/matches", &token, &match_body).await;
    assert_eq!(status, StatusCode::OK, "Should return OK for dedup");

    // Only one match
    let (_, json) = get(&app, "/api/matches", &token).await;
    assert_eq!(json["matches"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn test_logout_invalidates_session() {
    let (app, _dir) = build_test_app().await;
    let token = signup(&app, "hank", "pass").await;

    // Session works before logout
    let (status, _) = get(&app, &format!("/api/auth/session?token={}", token), "").await;
    assert_eq!(status, StatusCode::OK);

    // Logout
    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/logout")
        .header("x-session-token", &token)
        .body(Body::empty())
        .unwrap();
    let (status, _) = request(&app, req).await;
    assert_eq!(status, StatusCode::OK);

    // Session invalid after logout
    let (status, _) = get(&app, &format!("/api/auth/session?token={}", token), "").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_signup_validates_input() {
    let (app, _dir) = build_test_app().await;

    let body = serde_json::json!({"username": "", "password": "pass"});
    let req = Request::builder()
        .method("POST")
        .uri("/api/auth/signup")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();
    let (status, _) = request(&app, req).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

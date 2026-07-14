//! Google OAuth 2.0 sign-in / sign-up.
//!
//! Flow:
//!   1. GET /api/auth/google/login  → returns { url } → frontend redirects browser
//!   2. Google redirects to /api/auth/google/callback?code=…&state=…
//!   3. Backend exchanges code for tokens, fetches user info, creates/looks up
//!      user, creates a session, and redirects to the frontend with ?token=…

use axum::{
    extract::Query,
    response::{IntoResponse, Redirect},
};
use oauth2::{
    basic::BasicClient,
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope, TokenUrl,
    TokenResponse,
};
use serde::Deserialize;
use std::env;

use crate::auth;
use crate::db::DbPool;

// ── Types ───────────────────────────────────────────────────────────────────

/// The user info Google returns from the userinfo endpoint.
#[derive(Debug, Deserialize)]
struct GoogleUserInfo {
    sub: String,   // unique Google ID
    email: String,
    name: String,
}

/// Query params on the callback URL from Google.
#[derive(Debug, Deserialize)]
pub struct OAuthCallbackQuery {
    pub code: String,
    pub state: String,
}

// ── OAuth env reader ───────────────────────────────────────────────────────

fn google_env() -> (ClientId, ClientSecret, AuthUrl, TokenUrl, RedirectUrl) {
    let cid = ClientId::new(
        env::var("GOOGLE_CLIENT_ID").expect("GOOGLE_CLIENT_ID must be set"),
    );
    let cs = ClientSecret::new(
        env::var("GOOGLE_CLIENT_SECRET").expect("GOOGLE_CLIENT_SECRET must be set"),
    );
    let au = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
        .expect("Invalid Google auth URL");
    let tu = TokenUrl::new("https://oauth2.googleapis.com/token".to_string())
        .expect("Invalid Google token URL");
    let ru = RedirectUrl::new(
        env::var("GOOGLE_REDIRECT_URI").expect("GOOGLE_REDIRECT_URI must be set"),
    )
    .expect("Invalid GOOGLE_REDIRECT_URI");
    (cid, cs, au, tu, ru)
}

// ── Handlers ────────────────────────────────────────────────────────────────

/// Returns a Google OAuth URL for the frontend to redirect the user to.
pub async fn google_login() -> impl IntoResponse {
    let (cid, cs, au, tu, ru) = google_env();
    let client = BasicClient::new(cid)
        .set_client_secret(cs)
        .set_auth_uri(au)
        .set_token_uri(tu)
        .set_redirect_uri(ru);

    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .url();

    tracing::info!(
        "Google OAuth login initiated (state={})",
        csrf_token.secret()
    );

    axum::Json(serde_json::json!({
        "ok": true,
        "url": auth_url.to_string(),
        "state": csrf_token.secret().as_str(),
    }))
}

/// Handles the OAuth callback from Google.
pub async fn google_callback(
    Query(params): Query<OAuthCallbackQuery>,
    pool: &DbPool,
) -> impl IntoResponse {
    let frontend_url =
        env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".into());

    let http_client = reqwest::Client::new();

    // 1. Exchange code for tokens
    let (cid, cs, au, tu, ru) = google_env();
    let client = BasicClient::new(cid)
        .set_client_secret(cs)
        .set_auth_uri(au)
        .set_token_uri(tu)
        .set_redirect_uri(ru);

    let token_result = match client
        .exchange_code(AuthorizationCode::new(params.code))
        .request_async(&http_client)
        .await
    {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("OAuth token exchange failed: {}", e);
            return Redirect::to(&format!("{}/login?error=oauth_failed", frontend_url));
        }
    };

    // 2. Fetch user info from Google
    let userinfo: GoogleUserInfo = match http_client
        .get("https://www.googleapis.com/oauth2/v3/userinfo")
        .bearer_auth(token_result.access_token().secret())
        .send()
        .await
    {
        Ok(resp) => match resp.json().await {
            Ok(info) => info,
            Err(e) => {
                tracing::error!("Failed to parse userinfo: {}", e);
                return Redirect::to(&format!("{}/login?error=userinfo_failed", frontend_url));
            }
        },
        Err(e) => {
            tracing::error!("Userinfo request failed: {}", e);
            return Redirect::to(&format!("{}/login?error=userinfo_failed", frontend_url));
        }
    };

    tracing::info!(
        "Google OAuth callback: google_id={}, email={}, name={}",
        userinfo.sub,
        userinfo.email,
        userinfo.name,
    );

    // 3. Look up or create the user
    let user_id = find_or_create_user(pool, &userinfo).await;

    if user_id == 0 {
        tracing::error!("Failed to create/find user for google_id={}", userinfo.sub);
        return Redirect::to(&format!("{}/login?error=create_failed", frontend_url));
    }

    // 4. Create a session
    match auth::create_session(pool, user_id).await {
        Ok((_session_id, token)) => {
            tracing::info!("Created session for google user id={}", user_id);
            Redirect::to(&format!(
                "{}/oauth/callback?token={}",
                frontend_url, token
            ))
        }
        Err(e) => {
            tracing::error!("Failed to create session: {}", e);
            Redirect::to(&format!("{}/login?error=session_failed", frontend_url))
        }
    }
}

// ── User lookup / creation ──────────────────────────────────────────────────

async fn find_or_create_user(pool: &DbPool, info: &GoogleUserInfo) -> i64 {
    let conn = pool.conn().await;
    let sub = &info.sub;
    let email = &info.email;

    // 1. Try by google_id
    if let Ok(mut rows) = conn
        .query(
            "SELECT id FROM users WHERE google_id = ?1",
            libsql::params![sub.clone()],
        )
        .await
    {
        if let Ok(Some(row)) = rows.next().await {
            return row.get(0).unwrap_or(0);
        }
    }

    // 2. Try linking by email
    if let Ok(mut rows) = conn
        .query(
            "SELECT id FROM users WHERE email = ?1",
            libsql::params![email.clone()],
        )
        .await
    {
        if let Ok(Some(row)) = rows.next().await {
            let id: i64 = row.get(0).unwrap_or(0);
            let _ = conn
                .execute(
                    "UPDATE users SET google_id = ?1, auth_provider = 'google' WHERE id = ?2",
                    libsql::params![sub.clone(), id],
                )
                .await;
            tracing::info!("Linked Google account to existing user: id={}", id);
            return id;
        }
    }

    // 3. Create new user
    let username = make_username(&info.name);
    let insert = conn
        .execute(
            "INSERT INTO users (username, password, email, google_id, auth_provider) VALUES (?1, '', ?2, ?3, 'google')",
            libsql::params![username.clone(), email.clone(), sub.clone()],
        )
        .await;

    match insert {
        Ok(_) => {
            if let Ok(mut rows) = conn
                .query(
                    "SELECT id FROM users WHERE google_id = ?1",
                    libsql::params![sub.clone()],
                )
                .await
            {
                if let Ok(Some(row)) = rows.next().await {
                    let id: i64 = row.get(0).unwrap_or(0);
                    tracing::info!(
                        "Created new Google user: id={}, username={}",
                        id, username
                    );
                    return id;
                }
            }
            0
        }
        Err(e) => {
            tracing::warn!(
                "Username '{}' collision: {}. Retrying with suffix.",
                username, e
            );
            let username2 = make_username(&format!("{} {}", info.name, &sub[..8]));
            let _ = conn
                .execute(
                    "INSERT INTO users (username, password, email, google_id, auth_provider) VALUES (?1, '', ?2, ?3, 'google')",
                    libsql::params![username2, email.clone(), sub.clone()],
                )
                .await;
            if let Ok(mut rows) = conn
                .query(
                    "SELECT id FROM users WHERE google_id = ?1",
                    libsql::params![sub.clone()],
                )
                .await
            {
                if let Ok(Some(row)) = rows.next().await {
                    return row.get(0).unwrap_or(0);
                }
            }
            0
        }
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Derive a username from a Google display name.
fn make_username(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .take(30)
        .collect();
    if cleaned.is_empty() {
        "Player".to_string()
    } else {
        cleaned
    }
}

//! Authentication and session management for the Bridge Club API.
//!
//! Password: SHA-256 digest of UTF-8 bytes, base64 STANDARD encoded.
//! Session token: raw UUID v4 string.
//! Session DB id: SHA-256 digest of token string bytes, hex encoded (lowercase).
//! Session expiry: 30 days (Unix ms timestamp).

use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::db::{DbPool, SessionRow, UserRow};

/// Hash a password: SHA-256 of UTF-8 bytes → base64 STANDARD encode.
pub fn hash_password(password: &str) -> String {
    let hash = Sha256::digest(password.as_bytes());
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, hash)
}

/// Hash a session token: SHA-256 of token UTF-8 bytes → lowercase hex.
pub fn hash_token(token: &str) -> String {
    let hash = Sha256::digest(token.as_bytes());
    format!("{:x}", hash)
}

/// Generate a raw UUID v4 session token string.
pub fn generate_session_token() -> String {
    Uuid::new_v4().to_string()
}

/// Create a session row in the database with a 30-day expiry (Unix ms).
///
/// Returns the hashed session id (for DB storage) and the raw token to give to
/// the client.
pub async fn create_session(
    pool: &DbPool,
    user_id: i64,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    let token = generate_session_token();
    let session_id = hash_token(&token);

    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis() as i64;
    let expires_at = now_ms + 30 * 24 * 60 * 60 * 1000; // 30 days in ms

    let conn = pool.conn().await?;
    conn.execute(
        "INSERT INTO sessions (id, user_id, expires_at) VALUES (?1, ?2, ?3)",
        libsql::params![session_id.clone(), user_id, expires_at],
    )
    .await?;

    tracing::info!(
        "Created session for user_id={} (expires_at={})",
        user_id,
        expires_at
    );

    Ok((session_id, token))
}

/// Validate a session token. Returns the user row and session row if valid and
/// not expired.
///
/// If the session is within its expiry window, this returns `Some(…)`.
/// If the token is invalid, expired, or the DB lookup fails, returns `None`.
pub async fn validate_session(
    pool: &DbPool,
    token: &str,
) -> Result<Option<(UserRow, SessionRow)>, Box<dyn std::error::Error>> {
    let session_id = hash_token(token);
    let conn = pool.conn().await?;

    let mut rows = conn.query(
        "SELECT s.id, s.user_id, s.expires_at, u.id, u.username, u.password
         FROM sessions s
         JOIN users u ON u.id = s.user_id
         WHERE s.id = ?1",
        libsql::params![session_id],
    )
    .await?;

    match rows.next().await? {
        Some(row) => {
            let sid: String = row.get(0)?;
            let user_id: i64 = row.get(1)?;
            let expires_at: i64 = row.get(2)?;
            let uid: i64 = row.get(3)?;
            let username: String = row.get(4)?;
            let password: String = row.get(5)?;

            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_millis() as i64;

            if now_ms > expires_at {
                // Expired — clean it up
                conn.execute(
                    "DELETE FROM sessions WHERE id = ?1",
                    libsql::params![sid],
                )
                .await?;
                return Ok(None);
            }

            Ok(Some((
                UserRow {
                    id: uid,
                    username,
                    password,
                },
                SessionRow {
                    id: sid,
                    user_id,
                    expires_at,
                },
            )))
        }
        None => Ok(None),
    }
}

/// Delete a session from the database (logout).
pub async fn delete_session(
    pool: &DbPool,
    token: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let session_id = hash_token(token);
    let conn = pool.conn().await?;
    conn.execute(
        "DELETE FROM sessions WHERE id = ?1",
        libsql::params![session_id],
    )
    .await?;
    Ok(())
}

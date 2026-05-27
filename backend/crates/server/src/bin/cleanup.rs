//! One-time cleanup: delete test users, set hermes password.
//! Run once, then delete this file.

use libsql::Database;

const HERMES_PW: &str = "Ba5qCDOiumSqsQ7ZXVxplNVcqP0KCLCv/jltglVQkXU=";

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL required");
    let auth_token = std::env::var("DATABASE_AUTH_TOKEN").expect("DATABASE_AUTH_TOKEN required");

    let db = Database::open_remote(url, auth_token).unwrap();
    let conn = db.connect().unwrap();

    // Delete test users
    conn.execute(
        "DELETE FROM users WHERE id IN (16, 17, 18, 19, 20, 21)",
        libsql::params![],
    )
    .await
    .unwrap();
    println!("Deleted test users (ids 16-21)");

    // Set hermes password
    conn.execute(
        "UPDATE users SET password = ?1 WHERE username = 'hermes'",
        libsql::params![HERMES_PW],
    )
    .await
    .unwrap();
    println!("Set hermes password");

    // Also delete their sessions
    conn.execute(
        "DELETE FROM sessions WHERE user_id IN (16, 17, 18, 19, 20, 21)",
        libsql::params![],
    )
    .await
    .unwrap();
    println!("Deleted orphan sessions");

    println!("Cleanup complete!");
}

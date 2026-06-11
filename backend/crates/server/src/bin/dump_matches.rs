//! Dump all match data to JSON for migration.

use libsql::Builder;
use serde_json::json;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "file:local.db".into());
    let auth_token = env::var("DATABASE_AUTH_TOKEN").ok();

    println!("Connecting...");
    let db = if url.starts_with("libsql://") || url.starts_with("https://") {
        let token = auth_token.expect("DATABASE_AUTH_TOKEN required for remote");
        Builder::new_remote(url, token).build().await?
    } else {
        Builder::new_local(url).build().await?
    };
    let conn = db.connect()?;

    // Check tables
    let mut trows = conn.query(
        "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name",
        libsql::params![],
    ).await?;
    let mut tables = Vec::new();
    while let Some(row) = trows.next().await? {
        tables.push(row.get::<String>(0)?);
    }
    println!("Tables: {:?}", tables);

    // Count matches
    let mut crows = conn.query("SELECT COUNT(*) FROM matches", libsql::params![]).await?;
    if let Some(row) = crows.next().await? {
        println!("Match count: {}", row.get::<i64>(0)?);
    }

    // Dump matches
    let sql = "SELECT id, user_id, date, bot_difficulty, trump_suit, bet_size, bet_winner, partner, won_match, bet_winner_user_id, partner_user_id, winning_team, player1_sets, player2_sets, player3_sets, player4_sets, player1_hand, player2_hand, player3_hand, player4_hand, sets_data, players, players_int, room_id, elo_change, is_hidden, preview1, preview2, preview3, preview4 FROM matches ORDER BY id";
    
    println!("Running query...");
    let mut rows = conn.query(sql, libsql::params![]).await?;
    
    let mut matches = Vec::new();
    loop {
        match rows.next().await? {
            Some(row) => {
                matches.push(json!({
                    "id": row.get::<i64>(0).unwrap_or(0),
                    "user_id": row.get::<i64>(1).unwrap_or(0),
                    "date": row.get::<i64>(2).unwrap_or(0),
                    "bot_difficulty": row.get::<String>(3).unwrap_or_default(),
                    "trump_suit": row.get::<String>(4).unwrap_or_default(),
                    "bet_size": row.get::<i64>(5).unwrap_or(0),
                    "bet_winner": row.get::<i64>(6).unwrap_or(0),
                    "partner": row.get::<Option<i64>>(7).unwrap_or(None),
                    "won_match": row.get::<Option<i64>>(8).unwrap_or(None),
                    "bet_winner_user_id": row.get::<i64>(9).unwrap_or(0),
                    "partner_user_id": row.get::<i64>(10).unwrap_or(0),
                    "winning_team": row.get::<i64>(11).unwrap_or(1),
                    "player1_sets": row.get::<i64>(12).unwrap_or(0),
                    "player2_sets": row.get::<i64>(13).unwrap_or(0),
                    "player3_sets": row.get::<i64>(14).unwrap_or(0),
                    "player4_sets": row.get::<i64>(15).unwrap_or(0),
                    "player1_hand": row.get::<String>(16).unwrap_or_default(),
                    "player2_hand": row.get::<String>(17).unwrap_or_default(),
                    "player3_hand": row.get::<String>(18).unwrap_or_default(),
                    "player4_hand": row.get::<String>(19).unwrap_or_default(),
                    "sets_data": row.get::<Option<String>>(20).unwrap_or(None),
                    "players": row.get::<Option<String>>(21).unwrap_or(None),
                    "players_int": row.get::<i64>(22).unwrap_or(0),
                    "room_id": row.get::<Option<String>>(23).unwrap_or(None),
                    "elo_change": row.get::<i64>(24).unwrap_or(0),
                    "is_hidden": row.get::<i64>(25).unwrap_or(1),
                    "preview1": row.get::<Option<String>>(26).unwrap_or(None),
                    "preview2": row.get::<Option<String>>(27).unwrap_or(None),
                    "preview3": row.get::<Option<String>>(28).unwrap_or(None),
                    "preview4": row.get::<Option<String>>(29).unwrap_or(None),
                }));
            }
            None => break,
        }
    }

    // Dump match_participants
    let mut prows = conn.query(
        "SELECT rowid, match_id, user_id, seat_index FROM match_participants ORDER BY rowid",
        libsql::params![],
    ).await?;

    let mut participants = Vec::new();
    loop {
        match prows.next().await? {
            Some(row) => {
                participants.push(json!({
                    "id": row.get::<i64>(0).unwrap_or(0),
                    "match_id": row.get::<i64>(1).unwrap_or(0),
                    "user_id": row.get::<i64>(2).unwrap_or(0),
                    "seat_index": row.get::<i64>(3).unwrap_or(0),
                }));
            }
            None => break,
        }
    }

    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let output = json!({
        "matches": matches,
        "match_participants": participants,
        "exported_at": now,
    });

    let path = std::env::var("DUMP_PATH")
        .unwrap_or_else(|_| "/tmp/bridge_club_match_backup.json".to_string());
    std::fs::write(&path, serde_json::to_string_pretty(&output)?)?;

    println!("Dumped {} matches and {} participants to {}", matches.len(), participants.len(), path);
    Ok(())
}

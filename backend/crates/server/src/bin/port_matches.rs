//! Port old match data to the new schema.
//! Reads from the JSON backup, inserts into new tables.
//! Run: cargo run --bin port_matches
//! Run AFTER the new schema migration has created the new tables.

use libsql::Builder;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;

#[derive(Debug, Deserialize)]
struct OldMatch {
    id: i64,
    user_id: i64,
    date: i64,
    bot_difficulty: String,
    trump_suit: String,
    bet_size: i64,
    bet_winner: i64,
    partner: Option<i64>,
    won_match: Option<i64>,
    bet_winner_user_id: i64,
    partner_user_id: i64,
    winning_team: i64,
    player1_sets: i64,
    player2_sets: i64,
    player3_sets: i64,
    player4_sets: i64,
    player1_hand: String,
    player2_hand: String,
    player3_hand: String,
    player4_hand: String,
    sets_data: Option<String>,
    players: Option<String>,
    players_int: i64,
    room_id: Option<String>,
    elo_change: i64,
    is_hidden: i64,
    preview1: Option<String>,
    preview2: Option<String>,
    preview3: Option<String>,
    preview4: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OldParticipant {
    id: i64,
    match_id: i64,
    user_id: i64,
    seat_index: i64,
}

#[derive(Debug, Deserialize)]
struct Backup {
    matches: Vec<OldMatch>,
    #[allow(dead_code)]
    match_participants: Vec<OldParticipant>,
}

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    // Read backup
    let backup_path = env::var("DUMP_PATH")
        .unwrap_or_else(|_| "/tmp/bridge_club_match_backup.json".to_string());
    let backup_data = std::fs::read_to_string(&backup_path)?;
    let backup: Backup = serde_json::from_str(&backup_data)?;

    println!("Loaded {} matches from backup", backup.matches.len());

    // Connect to DB and run migration (rename old tables, create new ones)
    let url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "file:local.db".into());
    let auth_token = env::var("DATABASE_AUTH_TOKEN").ok();

    let db = if url.starts_with("libsql://") || url.starts_with("https://") {
        let token = auth_token.expect("DATABASE_AUTH_TOKEN required for remote");
        Builder::new_remote(url.clone(), token).build().await?
    } else {
        Builder::new_local(url).build().await?
    };
    let conn = db.connect()?;

    // Run migration: rename old tables, create new ones
    println!("Running schema migration...");
    let _ = conn.execute_batch("ALTER TABLE matches RENAME TO matches_old;").await;
    let _ = conn.execute_batch("ALTER TABLE match_participants RENAME TO match_participants_old;").await;

    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS matches (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            room_id         TEXT UNIQUE,
            created_at      INTEGER NOT NULL,
            trump_suit      TEXT NOT NULL,
            bet_size        INTEGER NOT NULL,
            bet_winner_idx  INTEGER NOT NULL,
            partner_idx     INTEGER,
            partner_card    TEXT,
            winning_team    INTEGER NOT NULL,
            team1_sets      INTEGER NOT NULL DEFAULT 0,
            team2_sets      INTEGER NOT NULL DEFAULT 0,
            sets_data       TEXT,
            match_type      TEXT NOT NULL DEFAULT 'multi',
            is_hidden       INTEGER NOT NULL DEFAULT 1
        );
        CREATE TABLE IF NOT EXISTS match_participants (
            id              INTEGER PRIMARY KEY AUTOINCREMENT,
            match_id        INTEGER NOT NULL REFERENCES matches(id) ON DELETE CASCADE,
            user_id         INTEGER NOT NULL REFERENCES users(id),
            seat_index      INTEGER NOT NULL CHECK(seat_index BETWEEN 0 AND 3),
            team            INTEGER NOT NULL CHECK(team IN (1, 2)),
            sets_won        INTEGER NOT NULL DEFAULT 0,
            cards_played    TEXT NOT NULL DEFAULT '[]',
            hand_preview    TEXT,
            elo_change      INTEGER NOT NULL DEFAULT 0,
            UNIQUE(match_id, seat_index)
        );
    ").await?;
    println!("Migration done.");

    // Build a map from username to user_id for player resolution
    let mut username_to_id: HashMap<String, i64> = HashMap::new();
    let mut urows = conn.query("SELECT id, username FROM users", libsql::params![]).await?;
    while let Some(row) = urows.next().await? {
        let id: i64 = row.get(0)?;
        let name: String = row.get(1)?;
        username_to_id.insert(name, id);
    }

    // Known bot IDs
    username_to_id.insert("Bot-Alpha".to_string(), 1);
    username_to_id.insert("Bot-Beta".to_string(), 2);
    username_to_id.insert("Bot-Gamma".to_string(), 3);

    let mut ported = 0;
    let mut skipped = 0;

    for om in &backup.matches {
        // Check if this match already exists in the new table (by id or room_id)
        let mut exists = false;
        if let Ok(mut rows) = conn.query(
            "SELECT id FROM matches WHERE id = ?1",
            libsql::params![om.id],
        ).await {
            if let Ok(Some(_)) = rows.next().await {
                exists = true;
            }
        }
        if exists {
            skipped += 1;
            continue;
        }

        // Determine match_type and partner_card
        let match_type = if om.bot_difficulty.is_empty() { "multi" } else { "single" };
        
        // Partner card was stored in sets_data or isn't directly available.
        // We leave it as NULL for ported matches.
        let partner_card: Option<String> = None;

        // Parse players JSON to resolve user_ids per seat
        let player_ids: [i64; 4] = if let Some(ref players_json) = om.players {
            // Try to parse new format
            if let Ok(arr) = serde_json::from_str::<Vec<serde_json::Value>>(players_json) {
                let mut ids = [0i64; 4];
                for (i, entry) in arr.iter().enumerate().take(4) {
                    let name = entry["username"].as_str().unwrap_or("");
                    ids[i] = username_to_id.get(name).copied().unwrap_or(0);
                }
                ids
            } else {
                // Fall back to players_int
                [
                    om.players_int & 0xFF,
                    (om.players_int >> 8) & 0xFF,
                    (om.players_int >> 16) & 0xFF,
                    (om.players_int >> 24) & 0xFF,
                ]
            }
        } else {
            // No players data at all - fall back to players_int
            [
                om.players_int & 0xFF,
                (om.players_int >> 8) & 0xFF,
                (om.players_int >> 16) & 0xFF,
                (om.players_int >> 24) & 0xFF,
            ]
        };

        // Determine team for each seat based on old schema
        // Old: bet_winner = seat 1-4 (1-indexed), partner = seat 1-4 or null
        // New: bet_winner_idx = 0-3, partner_idx = 0-3 or null
        let bet_winner_idx = (om.bet_winner.max(1).min(4) - 1);
        let partner_idx = om.partner.map(|p| (p.max(1).min(4) - 1));
        
        let team1_seats = [
            bet_winner_idx,
            partner_idx.unwrap_or(99),
        ];

        let per_seat_data: [(i64, i64, &str, &Option<String>); 4] = [
            (0, om.player1_sets, &om.player1_hand, &om.preview1),
            (1, om.player2_sets, &om.player2_hand, &om.preview2),
            (2, om.player3_sets, &om.player3_hand, &om.preview3),
            (3, om.player4_sets, &om.player4_hand, &om.preview4),
        ];

        // Compute team sets
        let mut team_sets = [0i64, 0i64];
        for (seat_idx, _sets, _hand, _prev) in &per_seat_data {
            let is_team1 = *seat_idx == bet_winner_idx || partner_idx.map_or(false, |p| *seat_idx == p);
            // We'll compute from participant data below
        }

        // Compute total team sets from individual
        let team1_sets: i64 = per_seat_data.iter()
            .filter(|(s, _, _, _)| *s == bet_winner_idx || partner_idx.map_or(false, |p| *s == p))
            .map(|(_, sets, _, _)| *sets)
            .sum();
        let team2_sets: i64 = per_seat_data.iter()
            .filter(|(s, _, _, _)| *s != bet_winner_idx && !partner_idx.map_or(false, |p| *s == p))
            .map(|(_, sets, _, _)| *sets)
            .sum();

        // Insert match with the OLD id to preserve references
        let _ = conn.execute(
            "INSERT INTO matches (id, room_id, created_at, trump_suit, bet_size, bet_winner_idx, \
             partner_idx, partner_card, winning_team, team1_sets, team2_sets, sets_data, match_type, is_hidden) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            libsql::params![
                om.id,
                om.room_id.clone(),
                om.date,
                om.trump_suit.clone(),
                om.bet_size,
                bet_winner_idx,
                partner_idx,
                partner_card,
                om.winning_team,
                team1_sets,
                team2_sets,
                om.sets_data.clone(),
                match_type.to_string(),
                om.is_hidden,
            ],
        ).await?;

        // Insert participants for each seat
        for (seat_i, seat_sets, hand, preview_opt) in &per_seat_data {
            let uid = player_ids[*seat_i as usize];
            if uid == 0 {
                continue; // Skip empty seats
            }
            let is_team1 = *seat_i == bet_winner_idx || partner_idx.map_or(false, |p| *seat_i == p);
            let team: i64 = if is_team1 { 1 } else { 2 };
            
            let preview = preview_opt.as_ref()
                .map(|s| s.clone())
                .unwrap_or_else(|| compact_hand_preview(hand));

            // elo_change: only the owner (user_id = match.user_id) had elo_change stored
            let elo_change = if uid == om.user_id { om.elo_change } else { 0 };

            let _ = conn.execute(
                "INSERT INTO match_participants (match_id, user_id, seat_index, team, sets_won, cards_played, hand_preview, elo_change) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                libsql::params![
                    om.id,
                    uid,
                    seat_i,
                    team,
                    seat_sets,
                    hand.clone(),
                    preview,
                    elo_change,
                ],
            ).await?;
        }

        ported += 1;
    }

    println!("Done. Ported: {}, Skipped (already exist): {}", ported, skipped);
    Ok(())
}

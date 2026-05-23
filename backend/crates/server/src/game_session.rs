use uuid::Uuid;

use game_core::{Call, Card, GamePhase, Table};

use crate::bot::{auto_decide_and_act, BotDifficulty};
use crate::session::GameRoom;

// ── SinglePlayerSession ────────────────────────────────────────────────────

/// Tracks a single-player game session (human vs 3 bots).
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SinglePlayerSession {
    pub room_id: Uuid,
    pub human_seat_index: usize,
    pub difficulty: BotDifficulty,
}

// ── Human action enum ──────────────────────────────────────────────────────

/// Actions a human player can take.
pub enum HumanAction {
    Call(Call),
    PlayCard(Card),
    SelectPartner(Card),
}

// ── Create a single-player game ────────────────────────────────────────────

/// Create a new single-player game: one human + three bots.
///
/// Returns the `GameRoom` and a `SinglePlayerSession` describing the game.
/// The caller is responsible for inserting the room into the shared state.
///
/// After this function returns, bots may still need to act if it's not the
/// human's turn to go first. Call `process_bot_turns` immediately after.
pub fn new_single_player_game(
    username: &str,
    difficulty: BotDifficulty,
) -> (GameRoom, SinglePlayerSession) {
    let mut room = GameRoom::new();
    let bot_names = ["Bot-Alpha", "Bot-Beta", "Bot-Gamma"];

    // Add human player (first player gets seat 0)
    let (_human_id, human_seat) = room
        .add_player(username)
        .expect("Failed to add human player to new room");

    // Add bot players
    for name in &bot_names {
        room.add_player(name)
            .expect("Failed to add bot player to new room");
    }

    // Deal the cards
    room.table.deal();
    room.is_started = true;

    let session = SinglePlayerSession {
        room_id: room.room_id,
        human_seat_index: human_seat,
        difficulty,
    };

    (room, session)
}

// ── Process bot turns ──────────────────────────────────────────────────────

/// Keep processing bot turns as long as the current player is a bot.
///
/// Call this after a human move to let all following bots act before
/// returning control to the human.
pub fn process_bot_turns(table: &mut Table, human_seat: usize, difficulty: BotDifficulty) {
    loop {
        // Stop if the game is finished
        if table.phase == GamePhase::Finished {
            break;
        }
        // Stop if we're in a phase with no bot actions
        if table.phase == GamePhase::Scoring || table.phase == GamePhase::Dealing {
            break;
        }

        let current = table.current_player_index();

        // If it's the human's turn, stop
        if current == human_seat {
            break;
        }

        // Run the bot
        if let Err(e) = auto_decide_and_act(table, difficulty) {
            tracing::warn!(
                "Bot error at phase {:?}, player {}: {}",
                table.phase,
                current,
                e
            );
            break;
        }
    }
}

// ── Human move + bot chain reaction ────────────────────────────────────────

/// Apply a human action and then run all subsequent bot turns.
///
/// Validates that it is the human's turn and the action is valid for the
/// current phase.
pub fn action_human_move(
    table: &mut Table,
    human_seat: usize,
    action: &HumanAction,
    difficulty: BotDifficulty,
) -> Result<(), &'static str> {
    // Verify it's the human's turn
    let current = table.current_player_index();
    if current != human_seat {
        return Err("Not your turn");
    }

    // Apply the action based on the current phase
    match (table.phase, action) {
        (GamePhase::Bidding, HumanAction::Call(call)) => {
            table.make_call(*call)?;
        }
        (GamePhase::PartnerSelection, HumanAction::SelectPartner(card)) => {
            table.select_partner(*card)?;
        }
        (GamePhase::Playing, HumanAction::PlayCard(card)) => {
            table.play_card(*card)?;
        }
        _ => {
            return Err("Invalid action for current game phase");
        }
    }

    // After the human acts, let bots take their turns
    process_bot_turns(table, human_seat, difficulty);

    Ok(())
}

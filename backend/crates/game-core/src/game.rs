use crate::bid::{AuctionState, Call, Contract};
use crate::card::Card;
use crate::deck::Deck;
use crate::player::{Direction, Player};
use crate::scoring::{DealScore, Trick, Vulnerability};
use serde::{Deserialize, Serialize};

// ── GamePhase ─────────────────────────────────────────────────────────────

/// Which phase of a single deal we're in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GamePhase {
    Dealing,
    Bidding,
    Playing,
    Scoring,
    Finished,
}

// ── Table ─────────────────────────────────────────────────────────────────

/// A full bridge table: 4 players, a deck, and the current deal state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    pub players: [Player; 4],
    pub vulnerability: Vulnerability,
    pub dealer: Direction,
    pub phase: GamePhase,
    pub deck: Deck,

    // Bidding state
    pub auction: Option<AuctionState>,

    // Play state
    pub contract: Option<Contract>,
    pub current_trick: Option<Vec<Card>>,  // cards played this trick
    pub completed_tricks: Vec<Trick>,
    pub ns_tricks: u8,
    pub ew_tricks: u8,

    // Scoring
    pub score: Option<DealScore>,
}

impl Table {
    /// Create a new table with named players.
    pub fn new(player_names: [&str; 4]) -> Self {
        let players = [
            Player::new(player_names[0]),
            Player::new(player_names[1]),
            Player::new(player_names[2]),
            Player::new(player_names[3]),
        ];

        Table {
            players,
            vulnerability: Vulnerability::None,
            dealer: Direction::North,
            phase: GamePhase::Dealing,
            deck: Deck::new(),
            auction: None,
            contract: None,
            current_trick: None,
            completed_tricks: Vec::new(),
            ns_tricks: 0,
            ew_tricks: 0,
            score: None,
        }
    }

    /// Shuffle and deal 13 cards to each player. Advances to Bidding phase.
    pub fn deal(&mut self) {
        // TODO:
        // 1. Shuffle deck
        // 2. Deal 13 cards to each player (alternating, or 1-by-1 bridge style)
        // 3. Set phase = Bidding
        // 4. Initialize auction with current dealer
        todo!("Table::deal")
    }

    /// Process a call from a player during the auction.
    pub fn make_call(&mut self, call: Call) -> Result<(), &'static str> {
        // TODO:
        // 1. Check phase == Bidding
        // 2. Forward call to auction.make_call()
        // 3. If auction ended, set phase = Playing, determine contract + declarer
        todo!("Table::make_call")
    }

    /// Play a card from the current player's hand.
    /// `card` is the card to play (the server should verify it belongs to
    /// the player who's on lead).
    pub fn play_card(&mut self, card: Card) -> Result<(), &'static str> {
        // TODO:
        // 1. Check phase == Playing
        // 2. Add card to current trick
        // 3. If 4 cards played, resolve trick (scoring::Trick::new)
        // 4. Update trick counts, advance to next trick
        // 5. If all 13 tricks played, set phase = Scoring, compute score
        todo!("Table::play_card")
    }

    /// Which player's turn it is (index 0..3).
    pub fn current_player_index(&self) -> usize {
        match self.phase {
            GamePhase::Bidding => {
                self.auction
                    .as_ref()
                    .map(|a| a.current_player)
                    .unwrap_or(0)
            }
            GamePhase::Playing => {
                // TODO: track declarer / dummy / next to play
                todo!("current_player_index during play")
            }
            _ => 0,
        }
    }

    /// Forward to the next deal (rotate dealer, increment board number, etc.).
    pub fn next_deal(&mut self) {
        // TODO: Reset phase to Dealing, rotate dealer, update vulnerability
        todo!("Table::next_deal")
    }
}

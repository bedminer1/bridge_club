use crate::bid::Contract;
use serde::{Deserialize, Serialize};

// ── Vulnerability ─────────────────────────────────────────────────────────

/// Whether a side is vulnerable (affects scoring multipliers).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Vulnerability {
    None,
    NorthSouth,
    EastWest,
    Both,
}

impl Vulnerability {
    pub fn is_vulnerable(&self, is_ns: bool) -> bool {
        match self {
            Vulnerability::None => false,
            Vulnerability::NorthSouth => is_ns,
            Vulnerability::EastWest => !is_ns,
            Vulnerability::Both => true,
        }
    }
}

// ── Trick ─────────────────────────────────────────────────────────────────

use crate::card::Card;
use crate::card::Suit;

/// The result of a single trick (4 cards played).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trick {
    /// Cards played in order (index 0 = lead, 1..3 = clockwise).
    pub cards: [Card; 4],
    /// Player index that won this trick.
    pub winner: usize,
    /// Suit that was led.
    pub lead_suit: Suit,
}

impl Trick {
    /// Create a new trick from the four played cards, given the trump suit
    /// (None = no trump). Returns the trick with the winner set.
    pub fn new(cards: [Card; 4], trump: Option<Suit>) -> Self {
        // TODO: determine lead suit from cards[0]
        // TODO: determine winner:
        //   - highest card of lead suit wins, UNLESS a trump was played
        //   - if trump played, highest trump wins
        todo!("Trick::new")
    }
}

// ── Score ─────────────────────────────────────────────────────────────────

/// The result of a completed contract (made or down).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContractResult {
    /// Contract made. Tricks taken above book.
    Made(u8),
    /// Contract defeated. Number of undertricks.
    Down(u8),
}

/// Computed score for one deal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DealScore {
    /// The contract that was played.
    pub contract: Contract,
    /// Result.
    pub result: ContractResult,
    /// Points awarded to the declaring side.
    pub declarer_points: i32,
    /// Points awarded to the defending side (usually 0 unless the contract
    /// went down — then it's the declaring side losing points, which is
    /// equivalent to the defending side gaining).
    pub defender_points: i32,
}

/// Compute the score for a completed deal under duplicate bridge scoring.
// This is a stub. Full duplicate bridge scoring is fairly involved (above/below
// line, slam bonuses, game bonuses, doubled/redoubled multipliers).
// Start with a simple scoring scheme, then expand.
pub fn score_deal(contract: &Contract, tricks_taken: u8, vulnerability: Vulnerability) -> DealScore {
    // TODO: Duplicate bridge scoring calculation.
    // 1. Determine if contract was made or down
    // 2. Calculate trick score (above/below the line)
    // 3. Calculate game / part-score / slam bonuses
    // 4. Apply double / redouble multipliers
    // 5. Calculate undertrick penalties
    todo!("scoring::score_deal")
}

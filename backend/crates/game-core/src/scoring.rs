use crate::card::Card;
use crate::card::Suit;
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

// ── Set (renamed from Trick) ───────────────────────────────────────────────

/// The result of a single set (4 cards played).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Set {
    /// Cards played in order (index 0 = lead, 1..3 = clockwise).
    pub cards: [Card; 4],
    /// Player index (0..3) that won this set.
    pub winner: usize,
    /// Suit that was led.
    pub lead_suit: Suit,
}

/// Returns true if card `c1` beats card `c2` under Singapore Bridge rules.
///
/// Priority:
/// 1. Same suit → higher rank wins.
/// 2. Trump beats non-trump.
/// 3. Led suit beats off-suit non-trump.
/// 4. Otherwise, c1 does not beat c2.
fn card1_beats(c1: Card, c2: Card, lead_suit: Suit, trump: Option<Suit>) -> bool {
    // 1. Same suit → higher rank wins
    if c1.suit == c2.suit {
        return c1.rank > c2.rank;
    }
    // 2. Trump beats non-trump
    if let Some(tr) = trump {
        if c1.suit == tr && c2.suit != tr {
            return true;
        }
        if c1.suit != tr && c2.suit == tr {
            return false;
        }
    }
    // 3. Led suit beats off-suit non-trump
    if c1.suit == lead_suit && c2.suit != lead_suit {
        return true;
    }
    false
}

impl Set {
    /// Create a new set from the four played cards, given the trump suit
    /// (None = no trump). Returns the set with the winner determined.
    pub fn new(cards: [Card; 4], trump: Option<Suit>) -> Self {
        let lead_suit = cards[0].suit;
        let mut winner = 0; // index into the cards array, not player index
        for i in 1..4 {
            if card1_beats(cards[i], cards[winner], lead_suit, trump) {
                winner = i;
            }
        }
        Set { cards, winner, lead_suit }
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
pub fn score_deal(_contract: &Contract, _tricks_taken: u8, _vulnerability: Vulnerability) -> DealScore {
    // TODO: Duplicate bridge scoring calculation.
    // 1. Determine if contract was made or down
    // 2. Calculate trick score (above/below the line)
    // 3. Calculate game / part-score / slam bonuses
    // 4. Apply double / redouble multipliers
    // 5. Calculate undertrick penalties
    todo!("scoring::score_deal")
}

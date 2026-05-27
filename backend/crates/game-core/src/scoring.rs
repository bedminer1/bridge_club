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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::{Card, Rank, Suit};

    fn c(suit: Suit, rank: Rank) -> Card {
        Card { suit, rank }
    }

    #[test]
    fn test_card1_beats_higher_rank_same_suit() {
        let c1 = c(Suit::Hearts, Rank::Ace);
        let c2 = c(Suit::Hearts, Rank::King);
        assert!(card1_beats(c1, c2, Suit::Hearts, None));
        assert!(!card1_beats(c2, c1, Suit::Hearts, None));
    }

    #[test]
    fn test_card1_beats_trump_beats_non_trump() {
        let trump_card = c(Suit::Spades, Rank::Two);
        let non_trump = c(Suit::Hearts, Rank::Ace);
        assert!(card1_beats(trump_card, non_trump, Suit::Hearts, Some(Suit::Spades)));
        assert!(!card1_beats(non_trump, trump_card, Suit::Hearts, Some(Suit::Spades)));
    }

    #[test]
    fn test_card1_beats_led_suit_beats_off_suit() {
        let led = c(Suit::Diamonds, Rank::Five);
        let off = c(Suit::Clubs, Rank::King);
        assert!(card1_beats(led, off, Suit::Diamonds, None));
        assert!(!card1_beats(off, led, Suit::Diamonds, None));
    }

    #[test]
    fn test_card1_beats_off_suit_loses_to_led_suit() {
        let led = c(Suit::Diamonds, Rank::Three);
        let off = c(Suit::Clubs, Rank::Ace);
        assert!(!card1_beats(off, led, Suit::Diamonds, None));
    }

    #[test]
    fn test_set_new_winner_highest_of_led_suit() {
        let cards = [
            c(Suit::Hearts, Rank::Two),   // lead: 2♥
            c(Suit::Hearts, Rank::King),  // K♥ beats 2♥
            c(Suit::Hearts, Rank::Five),  // 5♥ loses to K♥
            c(Suit::Hearts, Rank::Ten),   // 10♥ loses to K♥
        ];
        let set = Set::new(cards, None);
        assert_eq!(set.winner, 1); // K♥ wins
        assert_eq!(set.lead_suit, Suit::Hearts);
    }

    #[test]
    fn test_set_new_trump_wins_even_when_low() {
        let cards = [
            c(Suit::Hearts, Rank::Ace),   // lead: A♥
            c(Suit::Spades, Rank::Two),   // 2♠ trump! Beats A♥
            c(Suit::Hearts, Rank::King),  // K♥ loses to trump
            c(Suit::Hearts, Rank::Queen), // Q♥ loses to trump
        ];
        let set = Set::new(cards, Some(Suit::Spades));
        assert_eq!(set.winner, 1); // 2♠ wins
    }

    #[test]
    fn test_vulnerability_is_vulnerable() {
        assert!(!Vulnerability::None.is_vulnerable(true));
        assert!(!Vulnerability::None.is_vulnerable(false));
        assert!(Vulnerability::NorthSouth.is_vulnerable(true));
        assert!(!Vulnerability::NorthSouth.is_vulnerable(false));
        assert!(!Vulnerability::EastWest.is_vulnerable(true));
        assert!(Vulnerability::EastWest.is_vulnerable(false));
        assert!(Vulnerability::Both.is_vulnerable(true));
        assert!(Vulnerability::Both.is_vulnerable(false));
    }
}

use serde::{Deserialize, Serialize};
use std::fmt;

// ── Suit ─────────────────────────────────────────────────────────────────

/// The four suits in bridge. Ordering follows standard bridge:
/// Clubs < Diamonds < Hearts < Spades (alphabetical at the table, but for
/// bidding suit rank matters: ♣ < ♦ < ♥ < ♠, no NoTrump.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

impl Suit {
    /// All four suits in ascending order. Useful for iteration.
    pub const ALL: [Suit; 4] = [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades];

    /// Returns the suit symbol as a single Unicode char.
    pub fn symbol(&self) -> char {
        match self {
            Suit::Clubs => '♣',
            Suit::Diamonds => '♦',
            Suit::Hearts => '♥',
            Suit::Spades => '♠',
        }
    }

    /// Returns the single-letter abbreviation (C/D/H/S).
    pub fn abbrev(&self) -> char {
        match self {
            Suit::Clubs => 'C',
            Suit::Diamonds => 'D',
            Suit::Hearts => 'H',
            Suit::Spades => 'S',
        }
    }
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

// ── Rank ─────────────────────────────────────────────────────────────────

/// Card rank from Two (lowest) to Ace (highest).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Rank {
    // Numeric variants are ordered by their discriminant.
    Two = 2,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl Rank {
    pub const ALL: [Rank; 13] = [
        Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Six,
        Rank::Seven, Rank::Eight, Rank::Nine, Rank::Ten,
        Rank::Jack, Rank::Queen, Rank::King, Rank::Ace,
    ];

    /// Singapore Bridge point value: J=1, Q=2, K=3, A=4, other=0.
    pub fn points(&self) -> u8 {
        match self {
            Rank::Jack => 1,
            Rank::Queen => 2,
            Rank::King => 3,
            Rank::Ace => 4,
            _ => 0,
        }
    }

    /// Returns the short display string ("2".."10","J","Q","K","A").
    /// Note: numeric ranks return a `String` rather than `&'static str` because
    /// they're computed. For a truly static version, match each variant.
    pub fn abbrev(&self) -> String {
        match self {
            Rank::Jack => "J".into(),
            Rank::Queen => "Q".into(),
            Rank::King => "K".into(),
            Rank::Ace => "A".into(),
            other => (*other as u8).to_string(),
        }
    }
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.abbrev())
    }
}

// ── Card ─────────────────────────────────────────────────────────────────

/// A single playing card.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
}

impl Card {
    pub fn new(suit: Suit, rank: Rank) -> Self {
        Card { suit, rank }
    }

    /// Full human-readable string, e.g. "A♠", "10♥".
    pub fn to_unicode_string(&self) -> String {
        format!("{}{}", self.rank.abbrev(), self.suit.symbol())
    }

    /// Plain ASCII string, e.g. "AS", "10H".
    pub fn to_ascii_string(&self) -> String {
        format!("{}{}", self.rank.abbrev(), self.suit.abbrev())
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Default to Unicode display; consumers that need ASCII call to_ascii_string.
        write!(f, "{}", self.to_unicode_string())
    }
}

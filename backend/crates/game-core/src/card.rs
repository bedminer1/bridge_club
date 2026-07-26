//! Compact card representation: Suit=u8, Rank=u8, Card=packed u8, Hand=u64 bitfield.
use serde::{Deserialize, Serialize};
use std::fmt;

// ── Suit (u8) ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum Suit {
    Clubs = 0,
    Diamonds = 1,
    Hearts = 2,
    Spades = 3,
}

impl Suit {
    pub const ALL: [Suit; 4] = [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades];

    pub fn symbol(&self) -> char {
        match self {
            Suit::Clubs => '♣',
            Suit::Diamonds => '♦',
            Suit::Hearts => '♥',
            Suit::Spades => '♠',
        }
    }

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

// ── Rank (u8) ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum Rank {
    Two = 2, Three, Four, Five, Six, Seven, Eight, Nine, Ten, Jack, Queen, King, Ace,
}

impl Rank {
    pub const ALL: [Rank; 13] = [
        Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Six,
        Rank::Seven, Rank::Eight, Rank::Nine, Rank::Ten,
        Rank::Jack, Rank::Queen, Rank::King, Rank::Ace,
    ];

    pub fn points(&self) -> u8 {
        match self {
            Rank::Jack => 1, Rank::Queen => 2, Rank::King => 3, Rank::Ace => 4,
            _ => 0,
        }
    }

    pub fn abbrev(&self) -> String {
        match self {
            Rank::Jack => "J".into(), Rank::Queen => "Q".into(),
            Rank::King => "K".into(), Rank::Ace => "A".into(),
            other => (*other as u8).to_string(),
        }
    }
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.abbrev())
    }
}

// ── Card (packed u8) ──────────────────────────────────────────────────────

/// A single playing card packed into 1 byte.
/// Bits: [rank:5][suit:2] — 0..12 rank, 0..3 suit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(from = "SerdeCard", into = "SerdeCard")]
pub struct Card(u8);

/// Serde helper to preserve JSON shape: {"Suit":"Heart","Value":10,"Rank":"10"}
#[derive(Serialize, Deserialize)]
struct SerdeCard {
    #[serde(rename = "Suit")]
    suit: String,
    #[serde(rename = "Value")]
    value: u8,
    #[serde(rename = "Rank")]
    rank: String,
}

impl From<SerdeCard> for Card {
    fn from(s: SerdeCard) -> Self {
        let suit = match s.suit.as_str() {
            "Club" | "Clubs" => Suit::Clubs,
            "Diamond" | "Diamonds" => Suit::Diamonds,
            "Heart" | "Hearts" => Suit::Hearts,
            "Spades" | "Spade" => Suit::Spades,
            _ => Suit::Clubs,
        };
        let rank = match s.rank.as_str() {
            "2" => Rank::Two, "3" => Rank::Three, "4" => Rank::Four,
            "5" => Rank::Five, "6" => Rank::Six, "7" => Rank::Seven,
            "8" => Rank::Eight, "9" => Rank::Nine, "10" => Rank::Ten,
            "J" => Rank::Jack, "Q" => Rank::Queen, "K" => Rank::King,
            "A" => Rank::Ace, _ => Rank::Two,
        };
        Card::new(suit, rank)
    }
}

impl From<Card> for SerdeCard {
    fn from(c: Card) -> Self {
        SerdeCard {
            suit: match c.suit() {
                Suit::Clubs => "Club".into(),
                Suit::Diamonds => "Diamond".into(),
                Suit::Hearts => "Heart".into(),
                Suit::Spades => "Spades".into(),
            },
            value: c.rank() as u8,
            rank: c.rank().abbrev(),
        }
    }
}

impl Card {
    #[inline]
    pub const fn new(suit: Suit, rank: Rank) -> Self {
        Card(((rank as u8) << 2) | (suit as u8 & 0b11))
    }

    #[inline]
    pub fn suit(&self) -> Suit {
        match self.0 & 0b11 {
            0 => Suit::Clubs,
            1 => Suit::Diamonds,
            2 => Suit::Hearts,
            _ => Suit::Spades,
        }
    }

    #[inline]
    pub fn rank(&self) -> Rank {
        match (self.0 >> 2) & 0b1111 {
            2 => Rank::Two, 3 => Rank::Three, 4 => Rank::Four,
            5 => Rank::Five, 6 => Rank::Six, 7 => Rank::Seven,
            8 => Rank::Eight, 9 => Rank::Nine, 10 => Rank::Ten,
            11 => Rank::Jack, 12 => Rank::Queen, 13 => Rank::King,
            14 => Rank::Ace, _ => Rank::Two,
        }
    }

    pub fn to_unicode_string(&self) -> String {
        format!("{}{}", self.rank().abbrev(), self.suit().symbol())
    }

    pub fn to_ascii_string(&self) -> String {
        format!("{}{}", self.rank().abbrev(), self.suit().abbrev())
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_unicode_string())
    }
}

// ── Hand (u64 bitfield) ───────────────────────────────────────────────────

/// A bridge hand as a 52-bit bitfield. 1 bit per card in the deck.
/// Index: suit * 13 + rank_offset (Two=0..Ace=12).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hand(u64);

impl Hand {
    pub const fn empty() -> Self { Hand(0) }

    #[inline]
    pub fn has(&self, card: Card) -> bool {
        let idx = card.suit() as u8 as usize * 13 + (card.rank() as u8 - 2) as usize;
        (self.0 & (1u64 << idx)) != 0
    }

    #[inline]
    pub fn add(&mut self, card: Card) {
        let idx = card.suit() as u8 as usize * 13 + (card.rank() as u8 - 2) as usize;
        self.0 |= 1u64 << idx;
    }

    #[inline]
    pub fn remove(&mut self, card: Card) {
        let idx = card.suit() as u8 as usize * 13 + (card.rank() as u8 - 2) as usize;
        self.0 &= !(1u64 << idx);
    }

    pub fn len(&self) -> usize { self.0.count_ones() as usize }
    pub fn is_empty(&self) -> bool { self.0 == 0 }

    /// Iterate all cards in the hand.
    pub fn cards(&self) -> HandIter {
        HandIter { bits: self.0, pos: 0 }
    }

    pub fn to_vec(&self) -> Vec<Card> {
        self.cards().collect()
    }
}

pub struct HandIter { bits: u64, pos: u8 }

impl Iterator for HandIter {
    type Item = Card;
    fn next(&mut self) -> Option<Card> {
        while self.pos < 52 {
            if (self.bits >> self.pos) & 1 != 0 {
                let suit = Suit::ALL[(self.pos / 13) as usize];
                let rank = Rank::ALL[(self.pos % 13) as usize];
                self.pos += 1;
                return Some(Card::new(suit, rank));
            }
            self.pos += 1;
        }
        None
    }
}

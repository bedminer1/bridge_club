use crate::card::{Card, Suit};
use serde::{Deserialize, Serialize};

// ── Player ────────────────────────────────────────────────────────────────

/// A bridge player with a name and a hand of cards.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    pub hand: Vec<Card>,
}

impl Player {
    pub fn new(name: impl Into<String>) -> Self {
        Player {
            name: name.into(),
            hand: Vec::with_capacity(13),
        }
    }

    /// Add a card to the player's hand.
    pub fn receive_card(&mut self, card: Card) {
        self.hand.push(card);
    }

    /// Number of cards currently held.
    pub fn hand_size(&self) -> usize {
        self.hand.len()
    }

    /// Sort hand by suit then rank (bridge standard order).
    ///
    /// Suit order: Spades (first), Hearts, Clubs, Diamonds.
    /// Within each suit, ascending by rank (2 low, Ace high).
    pub fn sort_hand(&mut self) {
        // Suit sort order: Spades=0, Hearts=1, Clubs=2, Diamonds=3
        fn suit_order(suit: Suit) -> u8 {
            match suit {
                Suit::Spades => 0,
                Suit::Hearts => 1,
                Suit::Clubs => 2,
                Suit::Diamonds => 3,
            }
        }
        self.hand.sort_by_key(|c| (suit_order(c.suit), c.rank));
    }

    /// Remove and return a card from the hand. The `index` is the position
    /// in the hand (0..hand_size()).
    /// Panics if index is out of range.
    pub fn play_card(&mut self, index: usize) -> Card {
        self.hand.remove(index)
    }

    /// Does this player hold the given card?
    pub fn has_card(&self, card: &Card) -> bool {
        self.hand.contains(card)
    }

    /// Human-readable hand string, e.g. "♠AKQ ♥JT9 ♦8 ♣432"
    ///
    /// Cards are grouped by suit and displayed in descending rank order
    /// (Ace first) for readability.
    pub fn hand_string(&self) -> String {
        // Collect cards by suit in display order: Spades, Hearts, Clubs, Diamonds
        let mut spades = Vec::new();
        let mut hearts = Vec::new();
        let mut clubs = Vec::new();
        let mut diamonds = Vec::new();

        for card in &self.hand {
            match card.suit {
                Suit::Spades => spades.push(card),
                Suit::Hearts => hearts.push(card),
                Suit::Clubs => clubs.push(card),
                Suit::Diamonds => diamonds.push(card),
            }
        }

        // Sort each suit descending by rank (Ace first)
        fn sort_desc(cards: &mut Vec<&Card>) {
            cards.sort_by(|a, b| b.rank.cmp(&a.rank));
        }
        sort_desc(&mut spades);
        sort_desc(&mut hearts);
        sort_desc(&mut clubs);
        sort_desc(&mut diamonds);

        let mut parts = Vec::new();
        for (suit_sym, cards) in [
            ('♠', &spades),
            ('♥', &hearts),
            ('♣', &clubs),
            ('♦', &diamonds),
        ] {
            if !cards.is_empty() {
                let ranks: String = cards.iter().map(|c| c.rank.abbrev()).collect();
                parts.push(format!("{}{}", suit_sym, ranks));
            }
        }

        parts.join(" ")
    }
}

// ── Position / Direction ──────────────────────────────────────────────────

/// The four seats at a bridge table. Used for vulnerability, dealer, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    North = 0,
    East = 1,
    South = 2,
    West = 3,
}

impl Direction {
    pub const ALL: [Direction; 4] = [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ];

    /// Return the partner direction.
    pub fn partner(&self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::East => Direction::West,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
        }
    }

    /// Next player clockwise.
    pub fn next(&self) -> Direction {
        match self {
            Direction::North => Direction::East,
            Direction::East => Direction::South,
            Direction::South => Direction::West,
            Direction::West => Direction::North,
        }
    }

    /// Return (NS, EW) partnership grouping.
    pub fn is_north_south(&self) -> bool {
        matches!(self, Direction::North | Direction::South)
    }
}

use crate::card::Card;
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
    pub fn sort_hand(&mut self) {
        // TODO: sort by (suit, rank)
        todo!("Player::sort_hand")
    }

    /// Remove and return a card from the hand. The `index` is the position
    /// in the sorted hand (0..hand_size()).
    /// Panics if index is out of range.
    pub fn play_card(&mut self, index: usize) -> Card {
        // TODO: self.hand.swap_remove(index) or remove(index)
        todo!("Player::play_card")
    }

    /// Does this player hold the given card?
    pub fn has_card(&self, card: &Card) -> bool {
        self.hand.contains(card)
    }

    /// Human-readable hand string, e.g. "♠AKQ ♥JT9 ♦8 ♣432"
    pub fn hand_string(&self) -> String {
        // TODO: format grouped by suit
        todo!("Player::hand_string")
    }
}

// ── Position / Direction ─────────────────────────────────────────────────-

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

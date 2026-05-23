use crate::card::{Card, Rank, Suit};
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};

// ── Deck ─────────────────────────────────────────────────────────────────

/// A standard 52-card deck.
///
/// Designed for *deal-once* use: you construct a `Deck`, shuffle, and
/// repeatedly call `draw()` until it's empty. The deck owns its cards in a
/// private Vec and hands them out by value.
///
/// # Example
/// ```ignore
/// let mut deck = Deck::new();
/// deck.shuffle();
/// while deck.size() > 0 {
///     let card = deck.draw();
///     println!("{}", card);
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    /// Creates a fresh, unshuffled 52-card deck (Clubs 2..A, Diamonds 2..A, …).
    pub fn new() -> Self {
        let mut cards = Vec::with_capacity(52);
        for suit in Suit::ALL {
            for rank in Rank::ALL {
                cards.push(Card::new(suit, rank));
            }
        }
        Deck { cards }
    }

    /// Randomly permutes the remaining cards (Fisher-Yates shuffle).
    pub fn shuffle(&mut self) {
        let mut rng = thread_rng();
        self.cards.shuffle(&mut rng);
    }

    /// Removes and returns the top card.
    /// Panics if the deck is empty.
    pub fn draw(&mut self) -> Card {
        self.cards.pop().expect("deck empty")
    }

    /// Number of cards remaining in the deck.
    pub fn size(&self) -> usize {
        self.cards.len()
    }

    /// Returns true when all cards have been drawn.
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
}

impl Default for Deck {
    fn default() -> Self {
        Self::new()
    }
}

use crate::card::{Card, Rank, Suit};
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    pub fn new() -> Self {
        let mut cards = Vec::with_capacity(52);
        for suit in Suit::ALL {
            for rank in Rank::ALL {
                cards.push(Card::new(suit, rank));
            }
        }
        Deck { cards }
    }

    pub fn shuffle(&mut self) {
        let mut rng = thread_rng();
        self.cards.shuffle(&mut rng);
    }

    pub fn draw(&mut self) -> Card {
        self.cards.pop().expect("deck empty")
    }

    pub fn size(&self) -> usize { self.cards.len() }
    pub fn is_empty(&self) -> bool { self.cards.is_empty() }
}

impl Default for Deck {
    fn default() -> Self { Self::new() }
}

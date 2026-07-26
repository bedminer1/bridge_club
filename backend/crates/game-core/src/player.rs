use crate::card::{Card, Suit};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    pub hand: Vec<Card>,
}

impl Player {
    pub fn new(name: impl Into<String>) -> Self {
        Player { name: name.into(), hand: Vec::with_capacity(13) }
    }

    pub fn receive_card(&mut self, card: Card) { self.hand.push(card); }
    pub fn hand_size(&self) -> usize { self.hand.len() }

    pub fn sort_hand(&mut self) {
        fn suit_order(suit: Suit) -> u8 { suit as u8 }
        self.hand.sort_by_key(|c| (suit_order(c.suit()), c.rank()));
    }

    pub fn play_card(&mut self, index: usize) -> Card { self.hand.remove(index) }

    pub fn has_card(&self, card: &Card) -> bool {
        self.hand.iter().any(|c| c.suit() == card.suit() && c.rank() == card.rank())
    }

    pub fn hand_string(&self) -> String {
        let mut spades = Vec::new(); let mut hearts = Vec::new();
        let mut clubs = Vec::new(); let mut diamonds = Vec::new();
        for card in &self.hand {
            match card.suit() {
                Suit::Spades => spades.push(card),
                Suit::Hearts => hearts.push(card),
                Suit::Clubs => clubs.push(card),
                Suit::Diamonds => diamonds.push(card),
            }
        }
        fn sort_desc(cards: &mut Vec<&Card>) {
            cards.sort_by(|a, b| b.rank().cmp(&a.rank()));
        }
        sort_desc(&mut spades); sort_desc(&mut hearts);
        sort_desc(&mut clubs); sort_desc(&mut diamonds);
        let mut parts = Vec::new();
        for (suit_sym, cards) in [('♠', &spades), ('♥', &hearts), ('♣', &clubs), ('♦', &diamonds)] {
            if !cards.is_empty() {
                let ranks: String = cards.iter().map(|c| c.rank().abbrev()).collect();
                parts.push(format!("{}{}", suit_sym, ranks));
            }
        }
        parts.join(" ")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    North = 0, East = 1, South = 2, West = 3,
}

impl Direction {
    pub const ALL: [Direction; 4] = [Direction::North, Direction::East, Direction::South, Direction::West];
    pub fn partner(&self) -> Direction {
        match self { Direction::North => Direction::South, Direction::East => Direction::West,
            Direction::South => Direction::North, Direction::West => Direction::East }
    }
    pub fn next(&self) -> Direction {
        match self { Direction::North => Direction::East, Direction::East => Direction::South,
            Direction::South => Direction::West, Direction::West => Direction::North }
    }
    pub fn is_north_south(&self) -> bool { matches!(self, Direction::North | Direction::South) }
}

use crate::card::Card;
use crate::card::Suit;
use crate::bid::Contract;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Vulnerability { None, NorthSouth, EastWest, Both }

impl Vulnerability {
    pub fn is_vulnerable(&self, is_ns: bool) -> bool {
        match self { Vulnerability::None => false, Vulnerability::NorthSouth => is_ns,
            Vulnerability::EastWest => !is_ns, Vulnerability::Both => true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Set {
    pub cards: [Card; 4],
    pub winner: usize,
    pub lead_suit: Suit,
}

fn card1_beats(c1: Card, c2: Card, lead_suit: Suit, trump: Option<Suit>) -> bool {
    if c1.suit() == c2.suit() { return c1.rank() > c2.rank(); }
    if let Some(tr) = trump {
        if c1.suit() == tr && c2.suit() != tr { return true; }
        if c1.suit() != tr && c2.suit() == tr { return false; }
    }
    if c1.suit() == lead_suit && c2.suit() != lead_suit { return true; }
    false
}

impl Set {
    pub fn new(cards: [Card; 4], trump: Option<Suit>) -> Self {
        let lead_suit = cards[0].suit();
        let mut winner = 0;
        for i in 1..4 { if card1_beats(cards[i], cards[winner], lead_suit, trump) { winner = i; } }
        Set { cards, winner, lead_suit }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContractResult { Made(u8), Down(u8) }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DealScore {
    pub contract: Contract, pub result: ContractResult,
    pub declarer_points: i32, pub defender_points: i32,
}

pub fn score_deal(_contract: &Contract, _tricks_taken: u8, _vulnerability: Vulnerability) -> DealScore {
    todo!("scoring::score_deal")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::{Card, Rank, Suit};
    fn c(suit: Suit, rank: Rank) -> Card { Card::new(suit, rank) }

    #[test] fn test_card1_beats_higher_rank_same_suit() {
        assert!(card1_beats(c(Suit::Hearts, Rank::Ace), c(Suit::Hearts, Rank::King), Suit::Hearts, None)); }
    #[test] fn test_card1_beats_trump_beats_non_trump() {
        assert!(card1_beats(c(Suit::Spades, Rank::Two), c(Suit::Hearts, Rank::Ace), Suit::Hearts, Some(Suit::Spades))); }
    #[test] fn test_card1_beats_led_suit_beats_off_suit() {
        assert!(card1_beats(c(Suit::Diamonds, Rank::Five), c(Suit::Clubs, Rank::King), Suit::Diamonds, None)); }
    #[test] fn test_card1_beats_off_suit_loses_to_led_suit() {
        assert!(!card1_beats(c(Suit::Clubs, Rank::Ace), c(Suit::Diamonds, Rank::Three), Suit::Diamonds, None)); }
    #[test] fn test_set_new_winner_highest_of_led_suit() {
        let set = Set::new([c(Suit::Hearts, Rank::Two), c(Suit::Hearts, Rank::King),
            c(Suit::Hearts, Rank::Five), c(Suit::Hearts, Rank::Ten)], None);
        assert_eq!(set.winner, 1); }
    #[test] fn test_set_new_trump_wins_even_when_low() {
        let set = Set::new([c(Suit::Hearts, Rank::Ace), c(Suit::Spades, Rank::Two),
            c(Suit::Hearts, Rank::King), c(Suit::Hearts, Rank::Queen)], Some(Suit::Spades));
        assert_eq!(set.winner, 1); }
    #[test] fn test_vulnerability_is_vulnerable() {
        assert!(Vulnerability::NorthSouth.is_vulnerable(true)); }
}

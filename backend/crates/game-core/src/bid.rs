use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use crate::card::Suit;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Strain {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

impl Strain {
    pub fn from_suit(suit: Suit) -> Self {
        match suit { Suit::Clubs => Strain::Clubs, Suit::Diamonds => Strain::Diamonds,
            Suit::Hearts => Strain::Hearts, Suit::Spades => Strain::Spades }
    }
    pub fn symbol(&self) -> &'static str {
        match self { Strain::Clubs => "♣", Strain::Diamonds => "♦",
            Strain::Hearts => "♥", Strain::Spades => "♠" }
    }
}
impl fmt::Display for Strain { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.symbol()) } }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bid { pub level: u8, pub strain: Strain }

impl Bid {
    pub fn new(level: u8, strain: Strain) -> Self { Bid { level, strain } }
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim(); if s.len() < 2 { return None; }
        let level: u8 = s.chars().next()?.to_digit(10)? as u8;
        if !(1..=7).contains(&level) { return None; }
        let strain = match s[1..].trim().to_lowercase().as_str() {
            "c"|"♣"|"clubs" => Strain::Clubs, "d"|"♦"|"diamonds" => Strain::Diamonds,
            "h"|"♥"|"hearts" => Strain::Hearts, "s"|"♠"|"spades" => Strain::Spades,
            _ => return None,
        };
        Some(Bid { level, strain })
    }
}

fn strain_cmp(strain: Strain) -> u8 {
    match strain { Strain::Clubs => 0, Strain::Diamonds => 1, Strain::Hearts => 2, Strain::Spades => 3 }
}
impl PartialOrd for Bid {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> { Some(self.cmp(o)) }
}
impl Ord for Bid {
    fn cmp(&self, o: &Self) -> Ordering {
        match self.level.cmp(&o.level) { Ordering::Equal => strain_cmp(self.strain).cmp(&strain_cmp(o.strain)), other => other }
    }
}
impl fmt::Display for Bid { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}{}", self.level, self.strain) } }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Call { Bid(Bid), Pass }

impl Call {
    pub fn as_bid(&self) -> Option<Bid> { match self { Call::Bid(b) => Some(*b), _ => None } }
    pub fn abbreviation(&self) -> String { match self { Call::Bid(b) => b.to_string(), Call::Pass => "Pass".into() } }
}
impl fmt::Display for Call { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.abbreviation()) } }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Contract { pub bid: Bid, pub declarer: usize }

impl Contract {
    pub fn tricks_required(&self) -> u8 { self.bid.level + 6 }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuctionState {
    pub current_player: usize,
    pub last_bid: Option<Bid>,
    pub last_bidder: Option<usize>,
    pub consecutive_passes: u8,
    pub call_history: Vec<Call>,
}

impl AuctionState {
    pub fn new(dealer: usize) -> Self {
        AuctionState { current_player: dealer, last_bid: None, last_bidder: None,
            consecutive_passes: 0, call_history: Vec::new() }
    }
    pub fn make_call(&mut self, call: Call) -> Result<(), &'static str> {
        if self.is_ended() { return Err("Auction has ended"); }
        match call {
            Call::Bid(bid) => {
                if let Some(last) = self.last_bid { if bid <= last { return Err("Bid must outrank the current bid"); } }
                self.last_bid = Some(bid); self.last_bidder = Some(self.current_player); self.consecutive_passes = 0;
            }
            Call::Pass => { self.consecutive_passes += 1; }
        }
        self.call_history.push(call);
        self.current_player = (self.current_player + 1) % 4;
        Ok(())
    }
    pub fn is_ended(&self) -> bool {
        if self.call_history.len() < 4 { return false; }
        self.call_history[self.call_history.len()-3..].iter().all(|c| matches!(c, Call::Pass))
    }
    pub fn final_contract(&self) -> Option<Contract> {
        if !self.is_ended() { return None; }
        Some(Contract { bid: self.last_bid?, declarer: self.last_bidder? })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn test_bid_new() { let b = Bid::new(1, Strain::Clubs); assert_eq!(b.level, 1); }
    #[test] fn test_bid_ordering_same_level() {
        assert!(Bid::new(3, Strain::Clubs) < Bid::new(3, Strain::Diamonds));
        assert!(Bid::new(3, Strain::Diamonds) < Bid::new(3, Strain::Hearts));
        assert!(Bid::new(3, Strain::Hearts) < Bid::new(3, Strain::Spades)); }
    #[test] fn test_bid_ordering_different_level() { assert!(Bid::new(1, Strain::Spades) < Bid::new(2, Strain::Clubs)); }
    #[test] fn test_bid_parse_valid() { let b = Bid::parse("4hearts").unwrap(); assert_eq!(b.level, 4); }
    #[test] fn test_bid_parse_invalid() { assert!(Bid::parse("0hearts").is_none()); }
    #[test] fn test_bid_display() { assert!(Bid::new(4, Strain::Hearts).to_string().contains('4')); }
    #[test] fn test_call_abbreviation() { assert_eq!(Call::Bid(Bid::new(3, Strain::Spades)).abbreviation(), "3♠"); }
    #[test] fn test_call_as_bid() { assert!(Call::Bid(Bid::new(2, Strain::Diamonds)).as_bid().is_some()); }
    #[test] fn test_auction_new() { let a = AuctionState::new(0); assert!(!a.is_ended()); }
    #[test] fn test_auction_dealer_2() { assert_eq!(AuctionState::new(2).current_player, 2); }
    #[test] fn test_auction_bid_advances_player() { let mut a = AuctionState::new(0); a.make_call(Call::Bid(Bid::new(1, Strain::Clubs))).unwrap(); assert_eq!(a.current_player, 1); }
    #[test] fn test_auction_must_outrank() { let mut a = AuctionState::new(0); a.make_call(Call::Bid(Bid::new(2, Strain::Hearts))).unwrap(); assert!(a.make_call(Call::Bid(Bid::new(1, Strain::Spades))).is_err()); }
    #[test] fn test_auction_ends_after_3_passes() { let mut a = AuctionState::new(0); a.make_call(Call::Bid(Bid::new(1, Strain::Clubs))).unwrap(); for _ in 0..3 { a.make_call(Call::Pass).unwrap(); } assert!(a.is_ended()); }
    #[test] fn test_auction_no_moves_after_end() { let mut a = AuctionState::new(0); a.make_call(Call::Bid(Bid::new(1, Strain::Clubs))).unwrap(); a.make_call(Call::Pass).unwrap(); a.make_call(Call::Pass).unwrap(); a.make_call(Call::Pass).unwrap(); assert!(a.make_call(Call::Bid(Bid::new(2, Strain::Hearts))).is_err()); }
    #[test] fn test_auction_final_contract() { let mut a = AuctionState::new(1); a.make_call(Call::Pass).unwrap(); a.make_call(Call::Bid(Bid::new(2, Strain::Spades))).unwrap(); for _ in 0..3 { a.make_call(Call::Pass).unwrap(); } assert_eq!(a.final_contract().unwrap().bid, Bid::new(2, Strain::Spades)); }
    #[test] fn test_auction_all_pass_no_contract() { let mut a = AuctionState::new(0); for _ in 0..4 { a.make_call(Call::Pass).unwrap(); } assert!(a.final_contract().is_none()); }
    #[test] fn test_bid_display_roundtrip() { let b = Bid::new(5, Strain::Diamonds); let s = b.to_string(); assert!(Bid::parse(&s).is_some()); }
}

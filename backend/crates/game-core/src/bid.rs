use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;

use crate::card::Suit;

// ── Strain ───────────────────────────────────────────────────────────────

/// The five possible *strains* you can bid: Clubs, Diamonds, Hearts, Spades,
/// or No Trump. Ordering matches bridge convention (see `Bid::cmp`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Strain {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

impl Strain {
    /// Convert a `Suit` to `Strain` (Clubs→Clubs, etc.). This is *not*
    /// From<Suit> because the inverse isn't total.
    pub fn from_suit(suit: Suit) -> Self {
        match suit {
            Suit::Clubs => Strain::Clubs,
            Suit::Diamonds => Strain::Diamonds,
            Suit::Hearts => Strain::Hearts,
            Suit::Spades => Strain::Spades,
        }
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            Strain::Clubs => "♣",
            Strain::Diamonds => "♦",
            Strain::Hearts => "♥",
            Strain::Spades => "♠",
        }
    }
}

impl fmt::Display for Strain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

// ── Bid ───────────────────────────────────────────────────────────────────

/// A contract bid: level (1–7) + strain. E.g. `3NT`, `4♥`, `1♣`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bid {
    pub level: u8,   // 1..7
    pub strain: Strain,
}

impl Bid {
    /// Create a new bid. Does *not* validate level (callers should ensure
    /// 1 ≤ level ≤ 7).
    pub fn new(level: u8, strain: Strain) -> Self {
        Bid { level, strain }
    }

    /// Parse a bid string like "4hearts", "1C", "2♥".
    /// Level is the first char (digit 1-7). The rest is the strain.
    /// Returns None on parse failure.
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        if s.len() < 2 {
            return None;
        }
        // First character must be a digit 1-7
        let level_char = s.chars().next()?;
        let level: u8 = level_char.to_digit(10)? as u8;
        if !(1..=7).contains(&level) {
            return None;
        }
        let rest = &s[1..].trim();
        let strain = match rest.to_lowercase().as_str() {
            "c" | "♣" | "clubs" => Strain::Clubs,
            "d" | "♦" | "diamonds" => Strain::Diamonds,
            "h" | "♥" | "hearts" => Strain::Hearts,
            "s" | "♠" | "spades" => Strain::Spades,
            _ => return None,
        };
        Some(Bid { level, strain })
    }
}

/// Strain rank for comparison: Spades > Hearts > Diamonds > Clubs.
fn strain_cmp_rank(strain: Strain) -> u8 {
    match strain {
        Strain::Clubs => 0,
        Strain::Diamonds => 1,
        Strain::Hearts => 2,
        Strain::Spades => 3,
    }
}

/// Higher bids sort by level first, then by strain rank (NT > ♠ > ♥ > ♦ > ♣).
impl PartialOrd for Bid {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Bid {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.level.cmp(&other.level) {
            Ordering::Equal => {
                let self_rank = strain_cmp_rank(self.strain);
                let other_rank = strain_cmp_rank(other.strain);
                self_rank.cmp(&other_rank)
            }
            other => other,
        }
    }
}

impl fmt::Display for Bid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.level, self.strain)
    }
}

// ── Call ──────────────────────────────────────────────────────────────────

/// What a player can do on their turn during the auction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Call {
    /// Bid a contract (level + strain). Must outrank the previous bid.
    Bid(Bid),
    /// Pass.
    Pass,
}

impl Call {
    /// Returns the inner `Bid` if this call is a `Bid`, otherwise `None`.
    pub fn as_bid(&self) -> Option<Bid> {
        match self {
            Call::Bid(b) => Some(*b),
            _ => None,
        }
    }

    /// Human-readable short string: "Pass" or the bid itself.
    pub fn abbreviation(&self) -> String {
        match self {
            Call::Bid(b) => b.to_string(),
            Call::Pass => "Pass".into(),
        }
    }
}

impl fmt::Display for Call {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.abbreviation())
    }
}

// ── Contract ──────────────────────────────────────────────────────────────

/// The final contract after the auction ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Contract {
    pub bid: Bid,
    pub declarer: usize, // index of the declaring player (0..3)
}

impl Contract {
    /// Returns the number of sets the declaring side must take.
    /// Contract level + 6 (e.g. 4♥ → 10 tricks).
    pub fn tricks_required(&self) -> u8 {
        self.bid.level + 6
    }
}

// ── Auction State ─────────────────────────────────────────────────────────

/// Represents the current state of a bridge auction (bidding phase).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuctionState {
    /// Index (0..3) of the player whose turn it is to call.
    pub current_player: usize,
    /// The most recent bid (None if no bids yet).
    pub last_bid: Option<Bid>,
    /// The player index who made the last bid.
    pub last_bidder: Option<usize>,
    /// Number of consecutive passes since the last bid.
    pub consecutive_passes: u8,
    /// Full history of calls in order.
    pub call_history: Vec<Call>,
}

impl AuctionState {
    /// Create a fresh auction with dealer at index `dealer` (0..3).
    pub fn new(dealer: usize) -> Self {
        AuctionState {
            current_player: dealer,
            last_bid: None,
            last_bidder: None,
            consecutive_passes: 0,
            call_history: Vec::new(),
        }
    }

    /// Apply a call and advance the state.
    /// Returns Ok(()) if the call was legal, Err(&str) if illegal.
    ///
    /// Singapore Bridge rules:
    /// - Pass: increments consecutive_passes
    /// - Bid: must outrank last_bid, resets consecutive_passes
    pub fn make_call(&mut self, call: Call) -> Result<(), &'static str> {
        if self.is_ended() {
            return Err("Auction has ended");
        }
        match call {
            Call::Bid(bid) => {
                if let Some(last) = self.last_bid {
                    if bid <= last {
                        return Err("Bid must outrank the current bid");
                    }
                }
                self.last_bid = Some(bid);
                self.last_bidder = Some(self.current_player);
                self.consecutive_passes = 0;
            }
            Call::Pass => {
                self.consecutive_passes += 1;
            }
        }
        self.call_history.push(call);
        self.current_player = (self.current_player + 1) % 4;
        Ok(())
    }

    /// Returns true if the auction has ended.
    ///
    /// Singapore Bridge: auction ends when there are at least 4 calls and
    /// the last 3 are Pass (3 consecutive passes after the last raise).
    /// Also ends if 4 passes with no bid (everyone passed out).
    pub fn is_ended(&self) -> bool {
        if self.call_history.len() < 4 {
            return false;
        }
        // Check if the last 3 calls are all Pass
        let last_three = &self.call_history[self.call_history.len() - 3..];
        last_three.iter().all(|c| matches!(c, Call::Pass))
    }

    /// If the auction ended with a contract (not passed out), returns the
    /// final contract. Otherwise returns None.
    pub fn final_contract(&self) -> Option<Contract> {
        if !self.is_ended() {
            return None;
        }
        let bid = self.last_bid?;
        Some(Contract {
            bid,
            declarer: self.last_bidder?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Bid tests ─────────────────────────────────────────────────────────

    #[test]
    fn test_bid_new() {
        let b = Bid::new(1, Strain::Clubs);
        assert_eq!(b.level, 1);
        assert_eq!(b.strain, Strain::Clubs);
    }

    #[test]
    fn test_bid_ordering_same_level() {
        let clubs = Bid::new(3, Strain::Clubs);
        let diamonds = Bid::new(3, Strain::Diamonds);
        let hearts = Bid::new(3, Strain::Hearts);
        let spades = Bid::new(3, Strain::Spades);
        assert!(clubs < diamonds);
        assert!(diamonds < hearts);
        assert!(hearts < spades);
    }

    #[test]
    fn test_bid_ordering_different_level() {
        let low = Bid::new(1, Strain::Spades);
        let high = Bid::new(2, Strain::Clubs);
        assert!(low < high);
    }

    #[test]
    fn test_bid_parse_valid() {
        let b = Bid::parse("4hearts").unwrap();
        assert_eq!(b.level, 4);
        assert_eq!(b.strain, Strain::Hearts);

        let b = Bid::parse("1c").unwrap();
        assert_eq!(b.level, 1);
        assert_eq!(b.strain, Strain::Clubs);

        let b = Bid::parse("7S").unwrap();
        assert_eq!(b.level, 7);
        assert_eq!(b.strain, Strain::Spades);
    }

    #[test]
    fn test_bid_parse_invalid() {
        assert!(Bid::parse("0hearts").is_none());
        assert!(Bid::parse("8c").is_none());
        assert!(Bid::parse("abc").is_none());
        assert!(Bid::parse("").is_none());
    }

    #[test]
    fn test_bid_display() {
        let b = Bid::new(4, Strain::Hearts);
        let s = b.to_string();
        assert!(s.contains('4'));
        assert!(s.contains('♥') || s.contains("Hearts"));
    }

    // ── Call tests ────────────────────────────────────────────────────────

    #[test]
    fn test_call_abbreviation() {
        let bid = Call::Bid(Bid::new(3, Strain::Spades));
        assert_eq!(bid.abbreviation(), "3♠");
        assert_eq!(Call::Pass.abbreviation(), "Pass");
    }

    #[test]
    fn test_call_as_bid() {
        let bid = Call::Bid(Bid::new(2, Strain::Diamonds));
        assert!(bid.as_bid().is_some());
        assert_eq!(bid.as_bid().unwrap().level, 2);
        assert!(Call::Pass.as_bid().is_none());
    }

    // ── AuctionState tests ────────────────────────────────────────────────

    #[test]
    fn test_auction_new() {
        let a = AuctionState::new(0);
        assert_eq!(a.current_player, 0);
        assert!(a.last_bid.is_none());
        assert_eq!(a.consecutive_passes, 0);
        assert!(!a.is_ended());
    }

    #[test]
    fn test_auction_dealer_2() {
        let a = AuctionState::new(2);
        assert_eq!(a.current_player, 2);
    }

    #[test]
    fn test_auction_bid_advances_player() {
        let mut a = AuctionState::new(0);
        a.make_call(Call::Bid(Bid::new(1, Strain::Clubs))).unwrap();
        assert_eq!(a.current_player, 1);
        assert_eq!(a.last_bid, Some(Bid::new(1, Strain::Clubs)));
        assert_eq!(a.last_bidder, Some(0));
    }

    #[test]
    fn test_auction_must_outrank() {
        let mut a = AuctionState::new(0);
        a.make_call(Call::Bid(Bid::new(2, Strain::Hearts))).unwrap();
        // Can't bid lower
        assert!(a.make_call(Call::Bid(Bid::new(1, Strain::Spades))).is_err());
        // Can't bid same
        assert!(a.make_call(Call::Bid(Bid::new(2, Strain::Hearts))).is_err());
        // Can bid higher
        assert!(a.make_call(Call::Bid(Bid::new(2, Strain::Spades))).is_ok());
    }

    #[test]
    fn test_auction_ends_after_3_passes() {
        let mut a = AuctionState::new(0);
        a.make_call(Call::Bid(Bid::new(1, Strain::Clubs))).unwrap();
        assert!(!a.is_ended());
        a.make_call(Call::Pass).unwrap();
        assert!(!a.is_ended());
        a.make_call(Call::Pass).unwrap();
        assert!(!a.is_ended());
        a.make_call(Call::Pass).unwrap();
        assert!(a.is_ended());
    }

    #[test]
    fn test_auction_no_moves_after_end() {
        let mut a = AuctionState::new(0);
        a.make_call(Call::Bid(Bid::new(1, Strain::Clubs))).unwrap();
        a.make_call(Call::Pass).unwrap();
        a.make_call(Call::Pass).unwrap();
        a.make_call(Call::Pass).unwrap();
        assert!(a.is_ended());
        assert!(a.make_call(Call::Bid(Bid::new(2, Strain::Hearts))).is_err());
    }

    #[test]
    fn test_auction_final_contract() {
        let mut a = AuctionState::new(1);
        a.make_call(Call::Pass).unwrap();
        a.make_call(Call::Bid(Bid::new(2, Strain::Spades))).unwrap();
        a.make_call(Call::Pass).unwrap();
        a.make_call(Call::Pass).unwrap();
        a.make_call(Call::Pass).unwrap();
        let contract = a.final_contract().unwrap();
        assert_eq!(contract.bid, Bid::new(2, Strain::Spades));
        assert_eq!(contract.declarer, 2);
        assert_eq!(contract.tricks_required(), 8);
    }

    #[test]
    fn test_auction_all_pass_no_contract() {
        let mut a = AuctionState::new(0);
        a.make_call(Call::Pass).unwrap();
        a.make_call(Call::Pass).unwrap();
        a.make_call(Call::Pass).unwrap();
        assert!(!a.is_ended()); // only 3 calls, need 4
        a.make_call(Call::Pass).unwrap();
        assert!(a.is_ended());
        assert!(a.final_contract().is_none()); // passed out
    }

    #[test]
    fn test_bid_display_roundtrip() {
        let b = Bid::new(5, Strain::Diamonds);
        let s = b.to_string();
        let parsed = Bid::parse(&s);
        assert!(parsed.is_some());
        // Display uses symbols; parse handles symbols
        assert_eq!(parsed.unwrap(), b);
    }
}

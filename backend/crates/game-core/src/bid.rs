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

    /// Parse a bid string like "3NT", "4H", "1C", "2♥".
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

/// Strain rank for comparison: NT > Spades > Hearts > Diamonds > Clubs.
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
    /// Double the opponents' current contract. Only legal if the opponents
    /// made the last non-pass call.
    Double,
    /// Redouble a doubled contract. Only legal if we were doubled by
    /// the opponents.
    Redouble,
}

impl Call {
    /// Returns the inner `Bid` if this call is a `Bid`, otherwise `None`.
    pub fn as_bid(&self) -> Option<Bid> {
        match self {
            Call::Bid(b) => Some(*b),
            _ => None,
        }
    }

    /// Human-readable short string: "Pass", "X", "XX", or the bid itself.
    pub fn abbreviation(&self) -> String {
        match self {
            Call::Bid(b) => b.to_string(),
            Call::Pass => "Pass".into(),
            Call::Double => "X".into(),
            Call::Redouble => "XX".into(),
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
    pub doubled: bool,     // was the contract doubled?
    pub redoubled: bool,   // was it redoubled?
    pub declarer: usize,   // index of the declaring player (0..3)
}

impl Contract {
    /// Returns the number of tricks the declaring side must take.
    /// Contract level + 6 (e.g. 4♥ → 10 tricks).
    pub fn tricks_required(&self) -> u8 {
        self.bid.level + 6
    }

    /// Returns whether the contract is doubled or redoubled.
    pub fn is_doubled(&self) -> bool {
        self.doubled || self.redoubled
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
    /// Whether the current contract is doubled.
    pub doubled: bool,
    /// Which side doubled (true = NS doubled EW, false = EW doubled NS).
    pub doubling_side: Option<bool>,
    /// Whether doubled contract has been redoubled.
    pub redoubled: bool,
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
            doubled: false,
            doubling_side: None,
            redoubled: false,
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
            _ => return Err("Double/Redouble not allowed in Singapore Bridge auction"),
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
            doubled: self.doubled,
            redoubled: self.redoubled,
            declarer: self.last_bidder?,
        })
    }
}

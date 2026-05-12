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
    NoTrump,
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
            Strain::NoTrump => "NT",
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
        // TODO: validate level digit, parse strain from remaining chars
        todo!("Bid::parse")
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
        // TODO: compare level first, then strain
        todo!("Bid::cmp")
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

/// The final contract after the auction ends (3 passes after a bid).
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
    pub fn make_call(&mut self, call: Call) -> Result<(), &'static str> {
        // TODO: Validate and apply the call.
        // - Pass: increment consecutive_passes, check for auction end
        // - Bid: must be higher than last_bid, reset consecutive_passes
        // - Double: opponents must have made last bid, not already doubled
        // - Redouble: must be doubled by opponents, not already redoubled
        // Then advance current_player = (current_player + 1) % 4
        todo!("AuctionState::make_call")
    }

    /// Returns true if the auction has ended (3 consecutive passes after a bid,
    /// or 4 passes with no bid — passed out).
    pub fn is_ended(&self) -> bool {
        // TODO: logic for auction end conditions
        todo!("AuctionState::is_ended")
    }

    /// If the auction ended with a contract (not passed out), returns the
    /// final contract. Otherwise returns None.
    pub fn final_contract(&self) -> Option<Contract> {
        // TODO: determine declarer (first player on declaring side who bid
        // the strain of the final contract), build Contract struct
        todo!("AuctionState::final_contract")
    }
}

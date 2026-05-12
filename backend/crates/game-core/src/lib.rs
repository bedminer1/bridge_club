//! **game-core** — Pure bridge game logic.
//!
//! This crate contains zero I/O, zero async, zero networking.
//! It models the complete domain of contract bridge:
//! cards, decks, players, bidding, play, and scoring.
//!
//! Everything is serializable via serde so the `server` crate (or any
//! consumer) can send state over the wire without re-deriving.

pub mod bid;
pub mod card;
pub mod deck;
pub mod game;
pub mod player;
pub mod scoring;

// Re-exports for convenience.
pub use bid::{AuctionState, Bid, Call, Contract, Strain};
pub use card::{Card, Rank, Suit};
pub use deck::Deck;
pub use game::{GamePhase, Table};
pub use player::{Direction, Player};
pub use scoring::{ContractResult, DealScore, Trick, Vulnerability};

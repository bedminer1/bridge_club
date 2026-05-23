use crate::bid::{AuctionState, Call, Contract, Strain};
use crate::card::{Card, Suit};
use crate::deck::Deck;
use crate::player::{Direction, Player};
use crate::scoring::{DealScore, Set, Vulnerability};
use serde::{Deserialize, Serialize};

// ── GamePhase ─────────────────────────────────────────────────────────────

/// Which phase of a single deal we're in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GamePhase {
    Dealing,
    Bidding,
    PartnerSelection,
    Playing,
    Scoring,
    Finished,
}

// ── Table ─────────────────────────────────────────────────────────────────

/// A full bridge table: 4 players, a deck, and the current deal state.
///
/// Implements Singapore Bridge rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    pub players: [Player; 4],
    pub vulnerability: Vulnerability,
    pub dealer: Direction,
    pub phase: GamePhase,
    pub deck: Deck,

    // Bidding state
    pub auction: Option<AuctionState>,

    // Singapore Bridge bidding fields
    pub bet_size: u8,                // 0 = no bet yet, 1..7 after a bet
    pub trump_suit: Option<Suit>,    // trump suit for this round
    pub bet_winner: Option<usize>,   // index of the player who won the bet

    // Partner selection
    pub partner_idx: Option<usize>,  // index of the partner
    pub partner_card: Option<Card>,  // the card whose holder is the partner

    // Play state
    pub current_set_cards: Vec<Card>,
    pub completed_sets: Vec<Set>,
    pub sets_won: [u8; 4],          // sets won per player index
    pub lead_suit: Option<Suit>,     // suit led in the current set
    pub trump_played: bool,          // has trump been played in any set?
    pub current_player: usize,       // whose turn it is during play

    // Contract / scoring
    pub contract: Option<Contract>,
    pub score: Option<DealScore>,
}

impl Table {
    /// Create a new table with named players.
    pub fn new(player_names: [&str; 4]) -> Self {
        let players = [
            Player::new(player_names[0]),
            Player::new(player_names[1]),
            Player::new(player_names[2]),
            Player::new(player_names[3]),
        ];

        Table {
            players,
            vulnerability: Vulnerability::None,
            dealer: Direction::North,
            phase: GamePhase::Dealing,
            deck: Deck::new(),
            auction: None,
            bet_size: 0,
            trump_suit: None,
            bet_winner: None,
            partner_idx: None,
            partner_card: None,
            current_set_cards: Vec::new(),
            completed_sets: Vec::new(),
            sets_won: [0; 4],
            lead_suit: None,
            trump_played: false,
            current_player: 0,
            contract: None,
            score: None,
        }
    }

    /// Shuffle and deal 13 cards to each player. Advances to Bidding phase.
    ///
    /// P1 (index 0) starts the bidding.
    pub fn deal(&mut self) {
        let mut deck = Deck::new();
        deck.shuffle();

        // Clear all hands
        for player in &mut self.players {
            player.hand.clear();
        }

        // Deal 13 cards to each player (1-by-1 bridge style, 4×13 rounds)
        for i in 0..52 {
            let card = deck.draw();
            self.players[i % 4].receive_card(card);
        }

        // Sort each player's hand
        for player in &mut self.players {
            player.sort_hand();
        }

        // Reset all Singapore Bridge state
        self.bet_size = 0;
        self.trump_suit = None;
        self.bet_winner = None;
        self.partner_idx = None;
        self.partner_card = None;
        self.current_set_cards = Vec::new();
        self.completed_sets = Vec::new();
        self.sets_won = [0; 4];
        self.lead_suit = None;
        self.trump_played = false;
        self.current_player = 0;
        self.contract = None;
        self.score = None;

        // Start bidding phase — P1 (index 0) starts
        self.phase = GamePhase::Bidding;
        self.auction = Some(AuctionState::new(0));
        self.deck = deck; // store remaining deck (empty after deal)
    }

    /// Process a call from a player during the auction.
    ///
    /// Singapore Bridge rules:
    /// - Pass: 3 consecutive passes after a raise ends bidding.
    /// - Bid (raise): must outrank current bid (higher level, or same level
    ///   with higher suit priority: Spades > Hearts > Diamonds > Clubs).
    /// - If everyone passes, P1 (index 0) is forced to bet 1 Club.
    pub fn make_call(&mut self, call: Call) -> Result<(), &'static str> {
        if self.phase != GamePhase::Bidding {
            return Err("Not in bidding phase");
        }

        let auction = self.auction.as_mut().ok_or("No auction in progress")?;

        // Validate the call against auction rules
        match call {
            Call::Bid(bid) => {
                // Check bid is valid for Singapore Bridge (NoTrump not allowed)
                if bid.strain == Strain::NoTrump {
                    return Err("NoTrump not allowed in Singapore Bridge");
                }
                // Check bid outranks current
                if let Some(last) = auction.last_bid {
                    if bid <= last {
                        return Err("Bid must outrank current bid");
                    }
                }
                // Update Singapore Bridge state
                self.bet_size = bid.level;
                self.trump_suit = Some(match bid.strain {
                    Strain::Clubs => Suit::Clubs,
                    Strain::Diamonds => Suit::Diamonds,
                    Strain::Hearts => Suit::Hearts,
                    Strain::Spades => Suit::Spades,
                    _ => return Err("Invalid strain"),
                });
                self.bet_winner = Some(auction.current_player);
            }
            Call::Pass => {
                // Just a pass — no additional validation needed
            }
            _ => {
                return Err("Double/Redouble not supported in Singapore Bridge");
            }
        }

        auction.make_call(call)?;

        // Check if auction should end
        if auction.is_ended() {
            if auction.last_bid.is_none() {
                // Everyone passed — force P1 (index 0) to bet 1 Club
                self.bet_size = 1;
                self.trump_suit = Some(Suit::Clubs);
                self.bet_winner = Some(0);
            }

            // Build contract from the final bid
            if let Some(bid) = auction.last_bid {
                self.contract = Some(Contract {
                    bid,
                    doubled: false,
                    redoubled: false,
                    declarer: self.bet_winner.unwrap_or(0),
                });
            }

            // Transition to partner selection phase
            self.phase = GamePhase::PartnerSelection;
            // Bet winner gets to pick partner
            self.current_player = self.bet_winner.unwrap_or(0);
        }

        Ok(())
    }

    /// Select a partner card during PartnerSelection phase.
    ///
    /// The bet winner picks a card from the deck (not from their own hand).
    /// The holder of that card becomes the partner.
    ///
    /// Returns an error if:
    /// - Not in PartnerSelection phase
    /// - The card belongs to the bet winner's own hand
    /// - No bet winner is set
    pub fn select_partner(&mut self, card: Card) -> Result<(), &'static str> {
        if self.phase != GamePhase::PartnerSelection {
            return Err("Not in partner selection phase");
        }
        let bet_winner = self.bet_winner.ok_or("No bet winner set")?;

        // The card must not belong to the bet winner's own hand
        if self.players[bet_winner].has_card(&card) {
            return Err("Partner card cannot be from your own hand");
        }

        // Find which player holds this card
        let mut partner = None;
        for (i, player) in self.players.iter().enumerate() {
            if i != bet_winner && player.has_card(&card) {
                partner = Some(i);
                break;
            }
        }
        let partner_idx = partner.ok_or("Card not found in any player's hand")?;

        // Save partner info
        self.partner_card = Some(card);
        self.partner_idx = Some(partner_idx);

        // Transition to playing phase
        self.phase = GamePhase::Playing;
        self.current_set_cards = Vec::new();
        self.completed_sets = Vec::new();
        self.lead_suit = None;

        // Player to the left of bet winner leads the first trick
        self.current_player = (bet_winner + 1) % 4;

        Ok(())
    }

    /// Play a card from the current player's hand.
    ///
    /// Singapore Bridge rules:
    /// - Follow suit (must play led suit if you have it)
    /// - Cannot LEAD trump until trump has been played in a previous trick
    /// - After 4 cards, resolve the set, award it, check win condition
    pub fn play_card(&mut self, card: Card) -> Result<(), &'static str> {
        if self.phase != GamePhase::Playing {
            return Err("Not in playing phase");
        }

        let player_idx = self.current_player;
        let lead_suit = self.lead_suit;

        // Find the card in the player's hand (by suit+rank, not by Card equality)
        let card_idx = self.players[player_idx]
            .hand
            .iter()
            .position(|c| c.suit == card.suit && c.rank == card.rank)
            .ok_or("Card not found in player's hand")?;

        let played_card = card;

        // Rule: Follow suit — if player has the led suit, they must play it
        if let Some(ls) = lead_suit {
            if played_card.suit != ls {
                let has_led_suit = self.players[player_idx]
                    .hand
                    .iter()
                    .any(|c| c.suit == ls);
                if has_led_suit {
                    return Err("Must follow suit");
                }
            }
        }

        // Rule: Cannot LEAD trump before trump has been played
        // (Only applies when leading, i.e., first card of the set)
        if self.current_set_cards.is_empty() {
            if let Some(tr) = self.trump_suit {
                if played_card.suit == tr && !self.trump_played {
                    return Err("Cannot lead trump until trump has been played");
                }
            }
        }

        // Track the lead suit (first card of the set)
        if self.current_set_cards.is_empty() {
            self.lead_suit = Some(played_card.suit);
        }

        // Track whether trump has been played
        if self.trump_suit == Some(played_card.suit) {
            self.trump_played = true;
        }

        // Remove card from hand and add to current set
        self.players[player_idx].play_card(card_idx);
        self.current_set_cards.push(played_card);

        // Check if set is complete (4 cards played)
        if self.current_set_cards.len() == 4 {
            // Capture who led this set before resolving
            // Cards[0] was played by leader, cards[1] by (leader+1)%4, etc.
            // The leader was the player BEFORE the 4th card was played.
            // Since we advanced current_player after cards 0,1,2 but NOT after card 3,
            // current_player is still the player who played card 3.
            // So leader = (current_player + 1) % 4 for a 4-card set.
            let leader = (self.current_player + 1) % 4;

            let set_cards: [Card; 4] = [
                self.current_set_cards[0],
                self.current_set_cards[1],
                self.current_set_cards[2],
                self.current_set_cards[3],
            ];
            let set = Set::new(set_cards, self.trump_suit);

            // Map set winner index (0..3 within the 4 cards) to player index
            // Card at position `i` was played by (leader + i) % 4
            let actual_winner = (leader + set.winner) % 4;

            self.sets_won[actual_winner] += 1;
            // Store set with absolute winner index
            let mut stored_set = set;
            stored_set.winner = actual_winner;
            self.completed_sets.push(stored_set);
            self.current_set_cards = Vec::new();
            self.lead_suit = None;

            // Winner leads the next set
            self.current_player = actual_winner;

            // Check win condition
            self.check_win_condition();
        } else {
            // Advance to next player clockwise
            self.current_player = (self.current_player + 1) % 4;
        }

        Ok(())
    }

    /// Check whether either team has reached their winning target.
    ///
    /// Team1 (bet winner + partner): needs 6 + bet_size total sets.
    /// Team2 (the other two): needs 8 - bet_size total sets.
    fn check_win_condition(&mut self) {
        let bet_winner = match self.bet_winner {
            Some(bw) => bw,
            None => return,
        };
        let partner = self.partner_idx;

        // Team 1 sets = bet winner + partner
        let team1_sets = self.sets_won[bet_winner]
            + partner.map(|p| self.sets_won[p]).unwrap_or(0);
        let team1_target = 6 + self.bet_size;

        // Team 2 sets = the other two players
        let team2_sets: u8 = (0..4)
            .filter(|&i| i != bet_winner && partner.map_or(true, |p| i != p))
            .map(|i| self.sets_won[i])
            .sum();
        let team2_target = 8 - self.bet_size;

        if team1_sets >= team1_target || team2_sets >= team2_target {
            self.phase = GamePhase::Finished;
        }
    }

    /// Which player's turn it is (index 0..3).
    pub fn current_player_index(&self) -> usize {
        match self.phase {
            GamePhase::Bidding => {
                self.auction
                    .as_ref()
                    .map(|a| a.current_player)
                    .unwrap_or(0)
            }
            GamePhase::PartnerSelection => {
                // Bet winner picks the partner
                self.bet_winner.unwrap_or(0)
            }
            GamePhase::Playing => {
                self.current_player
            }
            _ => 0,
        }
    }

    /// Forward to the next deal (rotate dealer, reset phase).
    pub fn next_deal(&mut self) {
        // Rotate dealer
        self.dealer = match self.dealer {
            Direction::North => Direction::East,
            Direction::East => Direction::South,
            Direction::South => Direction::West,
            Direction::West => Direction::North,
        };

        // Reset to dealing phase
        self.phase = GamePhase::Dealing;
        self.auction = None;
        self.bet_size = 0;
        self.trump_suit = None;
        self.bet_winner = None;
        self.partner_idx = None;
        self.partner_card = None;
        self.current_set_cards = Vec::new();
        self.completed_sets = Vec::new();
        self.sets_won = [0; 4];
        self.lead_suit = None;
        self.trump_played = false;
        self.current_player = 0;
        self.contract = None;
        self.score = None;
    }
}

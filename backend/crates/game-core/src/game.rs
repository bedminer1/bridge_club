use crate::bid::{AuctionState, Call, Contract, Strain};
use crate::card::{Card, Suit};
use crate::deck::Deck;
use crate::player::{Direction, Player};
use crate::scoring::{DealScore, Set, Vulnerability};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GamePhase { Dealing, Bidding, PartnerSelection, Playing, Scoring, Finished }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    pub players: [Player; 4],
    pub vulnerability: Vulnerability,
    pub dealer: Direction,
    pub phase: GamePhase,
    pub deck: Deck,
    pub auction: Option<AuctionState>,
    pub bet_size: u8,
    pub trump_suit: Option<Suit>,
    pub bet_winner: Option<usize>,
    pub partner_idx: Option<usize>,
    pub partner_card: Option<Card>,
    pub current_set_cards: Vec<Card>,
    pub completed_sets: Vec<Set>,
    pub sets_won: [u8; 4],
    pub lead_suit: Option<Suit>,
    pub trump_played: bool,
    pub current_player: usize,
    pub contract: Option<Contract>,
    pub score: Option<DealScore>,
}

impl Table {
    pub fn new(player_names: [&str; 4]) -> Self {
        Table {
            players: [Player::new(player_names[0]), Player::new(player_names[1]),
                Player::new(player_names[2]), Player::new(player_names[3])],
            vulnerability: Vulnerability::None, dealer: Direction::North,
            phase: GamePhase::Dealing, deck: Deck::new(), auction: None,
            bet_size: 0, trump_suit: None, bet_winner: None,
            partner_idx: None, partner_card: None,
            current_set_cards: Vec::new(), completed_sets: Vec::new(),
            sets_won: [0; 4], lead_suit: None, trump_played: false,
            current_player: 0, contract: None, score: None,
        }
    }

    pub fn deal(&mut self) {
        let final_deck;
        loop {
            let mut deck = Deck::new(); deck.shuffle();
            for player in &mut self.players { player.hand.clear(); }
            for i in 0..52 { let card = deck.draw(); self.players[i % 4].receive_card(card); }
            let all_have_points = self.players.iter()
                .all(|p| p.hand.iter().map(|c| c.rank().points() as u32).sum::<u32>() >= 4);
            if all_have_points { final_deck = deck; break; }
        }
        for player in &mut self.players { player.sort_hand(); }
        self.bet_size = 0; self.trump_suit = None; self.bet_winner = None;
        self.partner_idx = None; self.partner_card = None;
        self.current_set_cards = Vec::new(); self.completed_sets = Vec::new();
        self.sets_won = [0; 4]; self.lead_suit = None;
        self.trump_played = false; self.current_player = 0;
        self.contract = None; self.score = None;
        self.phase = GamePhase::Bidding;
        self.auction = Some(AuctionState::new(0));
        self.deck = final_deck;
    }

    pub fn make_call(&mut self, call: Call) -> Result<(), &'static str> {
        if self.phase != GamePhase::Bidding { return Err("Not in bidding phase"); }
        let auction = self.auction.as_mut().ok_or("No auction in progress")?;
        match call {
            Call::Bid(bid) => {
                if let Some(last) = auction.last_bid { if bid <= last { return Err("Bid must outrank current bid"); } }
                self.bet_size = bid.level;
                self.trump_suit = Some(match bid.strain {
                    Strain::Clubs => Suit::Clubs, Strain::Diamonds => Suit::Diamonds,
                    Strain::Hearts => Suit::Hearts, Strain::Spades => Suit::Spades,
                });
                self.bet_winner = Some(auction.current_player);
            }
            Call::Pass => {}
        }
        auction.make_call(call)?;
        if auction.is_ended() {
            if auction.last_bid.is_none() {
                self.bet_size = 1; self.trump_suit = Some(Suit::Clubs); self.bet_winner = Some(0);
            }
            if let Some(bid) = auction.last_bid {
                self.contract = Some(Contract { bid, declarer: self.bet_winner.unwrap_or(0) });
            }
            self.phase = GamePhase::PartnerSelection;
            self.current_player = self.bet_winner.unwrap_or(0);
        }
        Ok(())
    }

    pub fn select_partner(&mut self, card: Card) -> Result<(), &'static str> {
        if self.phase != GamePhase::PartnerSelection { return Err("Not in partner selection phase"); }
        let bet_winner = self.bet_winner.ok_or("No bet winner set")?;
        if self.players[bet_winner].has_card(&card) {
            return Err("Partner card cannot be from your own hand");
        }
        let mut partner = None;
        for (i, player) in self.players.iter().enumerate() {
            if i != bet_winner && player.has_card(&card) { partner = Some(i); break; }
        }
        let partner_idx = partner.ok_or("Card not found in any player's hand")?;
        self.partner_card = Some(card); self.partner_idx = Some(partner_idx);
        self.phase = GamePhase::Playing;
        self.current_set_cards = Vec::new(); self.completed_sets = Vec::new();
        self.lead_suit = None;
        self.current_player = (bet_winner + 1) % 4;
        Ok(())
    }

    pub fn play_card(&mut self, card: Card) -> Result<(), &'static str> {
        if self.phase != GamePhase::Playing { return Err("Not in playing phase"); }
        let player_idx = self.current_player;
        let lead_suit = self.lead_suit;
        let card_idx = self.players[player_idx].hand.iter()
            .position(|c| c.suit() == card.suit() && c.rank() == card.rank())
            .ok_or("Card not found in player's hand")?;
        let played_card = card;
        if let Some(ls) = lead_suit {
            if played_card.suit() != ls {
                let has_led_suit = self.players[player_idx].hand.iter().any(|c| c.suit() == ls);
                if has_led_suit { return Err("Must follow suit"); }
            }
        }
        if self.current_set_cards.is_empty() {
            if let Some(tr) = self.trump_suit {
                if played_card.suit() == tr && !self.trump_played {
                    let has_non_trump = self.players[player_idx].hand.iter().any(|c| c.suit() != tr);
                    if has_non_trump { return Err("Cannot lead trump until trump has been played"); }
                }
            }
        }
        if self.current_set_cards.is_empty() { self.lead_suit = Some(played_card.suit()); }
        if self.trump_suit == Some(played_card.suit()) { self.trump_played = true; }
        self.players[player_idx].play_card(card_idx);
        self.current_set_cards.push(played_card);
        if self.current_set_cards.len() == 4 {
            let leader = (self.current_player + 1) % 4;
            let set_cards: [Card; 4] = [self.current_set_cards[0], self.current_set_cards[1],
                self.current_set_cards[2], self.current_set_cards[3]];
            let set = Set::new(set_cards, self.trump_suit);
            let actual_winner = (leader + set.winner) % 4;
            self.sets_won[actual_winner] += 1;
            let mut stored_set = set; stored_set.winner = actual_winner;
            self.completed_sets.push(stored_set);
            self.current_set_cards = Vec::new(); self.lead_suit = None;
            self.current_player = actual_winner;
            self.check_win_condition();
        } else {
            self.current_player = (self.current_player + 1) % 4;
        }
        Ok(())
    }

    fn check_win_condition(&mut self) {
        let bet_winner = match self.bet_winner { Some(bw) => bw, None => return };
        let partner = self.partner_idx;
        let team1_sets = self.sets_won[bet_winner] + partner.map(|p| self.sets_won[p]).unwrap_or(0);
        let team1_target = 6 + self.bet_size;
        let team2_sets: u8 = (0..4).filter(|&i| i != bet_winner && partner.map_or(true, |p| i != p))
            .map(|i| self.sets_won[i]).sum();
        let team2_target = 8 - self.bet_size;
        if team1_sets >= team1_target || team2_sets >= team2_target { self.phase = GamePhase::Finished; }
    }

    pub fn current_player_index(&self) -> usize {
        match self.phase {
            GamePhase::Bidding => self.auction.as_ref().map(|a| a.current_player).unwrap_or(0),
            GamePhase::PartnerSelection => self.bet_winner.unwrap_or(0),
            GamePhase::Playing => self.current_player,
            _ => 0,
        }
    }

    pub fn next_deal(&mut self) {
        self.dealer = match self.dealer {
            Direction::North => Direction::East, Direction::East => Direction::South,
            Direction::South => Direction::West, Direction::West => Direction::North,
        };
        self.phase = GamePhase::Dealing; self.auction = None;
        self.bet_size = 0; self.trump_suit = None; self.bet_winner = None;
        self.partner_idx = None; self.partner_card = None;
        self.current_set_cards = Vec::new(); self.completed_sets = Vec::new();
        self.sets_won = [0; 4]; self.lead_suit = None;
        self.trump_played = false; self.current_player = 0;
        self.contract = None; self.score = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bid::{Bid, Call, Strain};

    fn make_table() -> Table { Table::new(["Alice", "Bob", "Carol", "Dave"]) }

    #[test] fn test_table_new() { let t = make_table();
        assert_eq!(t.players[0].name, "Alice"); assert_eq!(t.phase, GamePhase::Dealing); }
    #[test] fn test_deal_gives_13_cards() { let mut t = make_table(); t.deal();
        for p in &t.players { assert_eq!(p.hand.len(), 13); } assert_eq!(t.phase, GamePhase::Bidding); }
    #[test] fn test_deal_each_player_has_points() { let mut t = make_table(); t.deal();
        for p in &t.players { let pts: u32 = p.hand.iter().map(|c| c.rank().points() as u32).sum();
            assert!(pts >= 4, "{} has {} points", p.name, pts); } }
    #[test] fn test_bidding_basic_raise() { let mut t = make_table(); t.deal();
        t.make_call(Call::Bid(Bid::new(2, Strain::Hearts))).unwrap();
        assert_eq!(t.bet_size, 2); assert_eq!(t.trump_suit, Some(Suit::Hearts)); }
    #[test] fn test_bidding_ends_with_3_passes() { let mut t = make_table(); t.deal();
        t.make_call(Call::Bid(Bid::new(2, Strain::Spades))).unwrap();
        for _ in 0..3 { t.make_call(Call::Pass).unwrap(); }
        assert_eq!(t.phase, GamePhase::PartnerSelection); }
    #[test] fn test_bidding_all_pass_forces_1_club() { let mut t = make_table(); t.deal();
        for _ in 0..4 { t.make_call(Call::Pass).unwrap(); }
        assert_eq!(t.phase, GamePhase::PartnerSelection);
        assert_eq!(t.bet_size, 1); assert_eq!(t.trump_suit, Some(Suit::Clubs)); }
    #[test] fn test_bidding_wrong_phase() { let mut t = make_table();
        assert!(t.make_call(Call::Bid(Bid::new(1, Strain::Clubs))).is_err()); }
    #[test] fn test_partner_selection_basic() { let mut t = make_table(); t.deal();
        for _ in 0..4 { t.make_call(Call::Pass).unwrap(); }
        let partner_card = t.players[1].hand.iter().find(|c| !t.players[0].has_card(c)).copied().unwrap();
        t.select_partner(partner_card).unwrap();
        assert_eq!(t.phase, GamePhase::Playing); assert!(t.partner_idx.unwrap() != 0); }
    #[test] fn test_partner_selection_cannot_use_own_card() { let mut t = make_table(); t.deal();
        for _ in 0..4 { t.make_call(Call::Pass).unwrap(); }
        let own = t.players[0].hand[0]; assert!(t.select_partner(own).is_err()); }
    #[test] fn test_next_deal_rotates_dealer() { let mut t = make_table();
        assert_eq!(t.dealer, Direction::North); t.next_deal(); assert_eq!(t.dealer, Direction::East); }
    #[test] fn test_next_deal_resets_phase() { let mut t = make_table(); t.deal();
        t.make_call(Call::Bid(Bid::new(2, Strain::Hearts))).unwrap(); t.next_deal();
        assert_eq!(t.phase, GamePhase::Dealing); }
    #[test] fn test_winner_detection_team1_reaches_target() { let mut t = make_table(); t.deal();
        t.make_call(Call::Bid(Bid::new(2, Strain::Spades))).unwrap();
        for _ in 0..3 { t.make_call(Call::Pass).unwrap(); }
        t.partner_idx = Some(2); t.phase = GamePhase::Playing;
        t.sets_won[0] = 4; t.sets_won[2] = 4; t.check_win_condition();
        assert_eq!(t.phase, GamePhase::Finished); }
    #[test] fn test_winner_detection_team2_reaches_target() { let mut t = make_table(); t.deal();
        t.make_call(Call::Bid(Bid::new(1, Strain::Clubs))).unwrap();
        for _ in 0..3 { t.make_call(Call::Pass).unwrap(); }
        t.partner_idx = Some(2); t.phase = GamePhase::Playing;
        t.sets_won[1] = 4; t.sets_won[3] = 3; t.check_win_condition();
        assert_eq!(t.phase, GamePhase::Finished); }
}

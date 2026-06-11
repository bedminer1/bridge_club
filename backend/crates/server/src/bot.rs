use game_core::{
    Bid, Call, Card, GamePhase, Rank, Strain, Suit, Table,
};

// ── Difficulty ─────────────────────────────────────────────────────────────

/// Bot difficulty level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BotDifficulty {
    Easy,
    Medium,
}

// ── Card comparison ────────────────────────────────────────────────────────

/// Returns true if card `a` beats card `b` under Singapore Bridge rules.
///
/// Priority:
/// 1. Same suit — higher rank wins.
/// 2. Trump beats non-trump.
/// 3. Led suit beats off-suit non-trump.
/// 4. Otherwise, `a` does not beat `b`.
pub fn card_beats(
    a: &Card,
    b: &Card,
    trump: Option<Suit>,
    lead_suit: Option<Suit>,
) -> bool {
    // 1. Same suit — higher rank wins
    if a.suit == b.suit {
        return a.rank > b.rank;
    }
    // 2. Trump beats non-trump
    if let Some(tr) = trump {
        if a.suit == tr && b.suit != tr {
            return true;
        }
        if a.suit != tr && b.suit == tr {
            return false;
        }
    }
    // 3. Led suit beats off-suit non-trump
    if let Some(ls) = lead_suit {
        if a.suit == ls && b.suit != ls {
            return true;
        }
    }
    false
}

/// Card ordering within a trick context. Trump > led suit > off-suit.
/// Within same suit, higher rank is stronger.
fn card_strength(card: &Card, trump: Option<Suit>, lead_suit: Option<Suit>) -> u16 {
    let suit_rank = if Some(card.suit) == trump {
        3u16
    } else if Some(card.suit) == lead_suit {
        2u16
    } else {
        1u16
    };
    // Rank value: Two=2 ... Ace=14
    let rank_val = card.rank as u16;
    (suit_rank << 8) | rank_val
}

/// Return the index of the strongest card in `cards` (for the trick context).
fn strongest_card_index<'a>(
    cards: &[&'a Card],
    trump: Option<Suit>,
    lead_suit: Option<Suit>,
) -> usize {
    cards
        .iter()
        .enumerate()
        .max_by_key(|(_, c)| card_strength(c, trump, lead_suit))
        .map(|(i, _)| i)
        .unwrap_or(0)
}

/// Return the index of the weakest card in `cards` (for the trick context).
fn weakest_card_index<'a>(
    cards: &[&'a Card],
    trump: Option<Suit>,
    lead_suit: Option<Suit>,
) -> usize {
    cards
        .iter()
        .enumerate()
        .min_by_key(|(_, c)| card_strength(c, trump, lead_suit))
        .map(|(i, _)| i)
        .unwrap_or(0)
}

// ── Legal moves ────────────────────────────────────────────────────────────

/// Returns the cards the current player can legally play during the playing phase.
fn legal_plays(table: &Table) -> Vec<Card> {
    let player_idx = table.current_player;
    let hand = &table.players[player_idx].hand;

    // If not leading: must follow suit if possible
    if !table.current_set_cards.is_empty() {
        if let Some(ls) = table.lead_suit {
            let follow_suit_cards: Vec<Card> = hand
                .iter()
                .filter(|c| c.suit == ls)
                .copied()
                .collect();
            if !follow_suit_cards.is_empty() {
                return follow_suit_cards;
            }
        }
    }

    // If leading: cannot lead trump before trump has been played
    if table.current_set_cards.is_empty() {
        if let Some(tr) = table.trump_suit {
            if !table.trump_played {
                return hand
                    .iter()
                    .filter(|c| c.suit != tr)
                    .copied()
                    .collect();
            }
        }
    }

    // Otherwise, play any card
    hand.clone()
}

// ── Suit scoring for bidding ───────────────────────────────────────────────

/// Compute a score for a suit based on card count and picture values.
/// Used by the bot to decide what suit to bid.
fn suit_score(hand: &[Card], suit: Suit) -> u8 {
    let suit_cards: Vec<&Card> = hand.iter().filter(|c| c.suit == suit).collect();
    let card_count = suit_cards.len() as u8;
    let picture_sum: u8 = suit_cards
        .iter()
        .map(|c| match c.rank {
            Rank::Jack => 1,
            Rank::Queen => 2,
            Rank::King => 3,
            Rank::Ace => 4,
            _ => 0,
        })
        .sum();
    (card_count * 2) + picture_sum
}

// ── Bidding decision ───────────────────────────────────────────────────────

/// Decide what call to make during the bidding phase.
fn decide_bid(table: &Table) -> Call {
    let player_idx = table.current_player_index();
    let hand = &table.players[player_idx].hand;

    // Score each suit
    let suits = [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades];
    let best_suit = suits
        .iter()
        .max_by_key(|s| suit_score(hand, **s))
        .copied()
        .unwrap_or(Suit::Clubs);

    let score = suit_score(hand, best_suit);

    let bet_size = if score >= 16 {
        3u8
    } else if score >= 13 {
        2u8
    } else {
        1u8
    };

    let strain = match best_suit {
        Suit::Clubs => Strain::Clubs,
        Suit::Diamonds => Strain::Diamonds,
        Suit::Hearts => Strain::Hearts,
        Suit::Spades => Strain::Spades,
    };

    let proposed_bid = Bid::new(bet_size, strain);

    // Check if this bid outranks the current bid
    let can_raise = match &table.auction {
        Some(auction) => match auction.last_bid {
            Some(last) => proposed_bid > last,
            None => true, // No current bid, can always bid
        },
        None => true,
    };

    if can_raise {
        Call::Bid(proposed_bid)
    } else {
        Call::Pass
    }
}

// ── Partner selection ──────────────────────────────────────────────────────

/// Decide which partner card to select during PartnerSelection phase.
fn decide_partner_card(table: &Table) -> Card {
    let bet_winner = table.bet_winner.unwrap_or(0);

    // Find the non-bet-winner player with the highest trump card
    let trump = table.trump_suit;
    let mut best_card: Option<Card> = None;
    let mut best_rank = 0u8; // lower than Two (rank 2)

    for (i, player) in table.players.iter().enumerate() {
        if i == bet_winner {
            continue;
        }
        for card in &player.hand {
            if let Some(tr) = trump {
                if card.suit == tr {
                    let r = card.rank as u8;
                    if r > best_rank {
                        best_rank = r;
                        best_card = Some(*card);
                    }
                }
            } else {
                // No trump suit — pick highest rank card overall
                let r = card.rank as u8;
                if r > best_rank {
                    best_rank = r;
                    best_card = Some(*card);
                }
            }
        }
    }

    // If we found a trump card (or high card), use it
    if let Some(card) = best_card {
        return card;
    }

    // Fallback: pick the next player clockwise and use their first card
    let fallback_idx = (bet_winner + 1) % 4;
    if let Some(first_card) = table.players[fallback_idx].hand.first() {
        return *first_card;
    }

    // Last resort: should never happen (everyone has cards after deal)
    Card::new(Suit::Clubs, Rank::Two)
}

// ── Team Model ─────────────────────────────────────────────────────────────

/// Maps out the two teams (bots and humans) and tracks observed feeding behavior.
///
/// After partner selection, teams are known exactly (`bet_winner` + `partner_idx`
/// vs the other two). This struct also tracks **feeding observations** — how often
/// each player appears to feed another player by dumping their weakest card when
/// that player was winning. These observations cross-validate the known teams and
/// help the bot decide which opponent is the real threat.
#[derive(Debug, Clone, Default)]
struct TeamModel {
    /// Which player index this bot is (0..3).
    pub bot_idx: usize,
    /// My known partner (None = not yet selected / no partner).
    pub my_partner: Option<usize>,
    /// The bet winner (always known after bidding).
    pub bet_winner: Option<usize>,
    /// Feed matrix: feed_counts[a][b] = how many times player a played their
    /// weakest card when player b was winning. High values suggest a is b's partner.
    pub feed_counts: [[u8; 4]; 4],
    /// Number of tricks fully completed (for normalizing percentages).
    pub tricks_observed: u8,
}

#[allow(dead_code)]
impl TeamModel {
    pub fn new(bot_idx: usize) -> Self {
        TeamModel {
            bot_idx,
            ..Default::default()
        }
    }

    /// Update the model from the current table state.
    pub fn observe(&mut self, table: &Table) {
        self.bet_winner = table.bet_winner;
        self.my_partner = table.partner_idx;

        // Replay completed sets to track feeding behavior
        self.feed_counts = [[0u8; 4]; 4];
        self.tricks_observed = 0;

        if table.completed_sets.is_empty() {
            return;
        }

        // Track ALL cards played before the current trick
        let mut played_so_far: Vec<Card> = Vec::new();

        let mut leader = (table.bet_winner.unwrap_or(0) + 1) % 4;
        for set in &table.completed_sets {
            self.tricks_observed += 1;

            // For each non-winner, check if they fed the winner
            for (i, card) in set.cards.iter().enumerate() {
                let player = (leader + i) % 4;
                if player == set.winner {
                    continue;
                }

                // Only count feeding if the player followed suit
                // (if they were void, they had no choice)
                if card.suit != set.lead_suit {
                    continue;
                }

                // Get the winner's card in the led suit
                let winner_card = &set.cards[set.winner % set.cards.len()];

                // If the played card already beats the winner, they weren't feeding
                if card_beats(card, winner_card, table.trump_suit, Some(set.lead_suit)) {
                    continue;
                }

                // Now check: was there ANY card stronger than the winner's card
                // in the led suit, that was still unplayed before this trick?
                let strongest_remaining = strongest_unplayed_in_suit(
                    &played_so_far,
                    &set.cards,
                    set.lead_suit,
                );

                match strongest_remaining {
                    Some(highest_unplayed) => {
                        // If the strongest unplayed card is higher rank than the winner's card,
                        // someone COULD have beaten the winner → possible feeding
                        if highest_unplayed.rank > winner_card.rank {
                            // The player chose not to play that stronger card → feeding
                            self.feed_counts[player][set.winner] =
                                self.feed_counts[player][set.winner].saturating_add(1);
                        }
                        // Otherwise highest unplayed ≤ winner_card → nobody could beat winner
                        // → not feeding
                    }
                    None => {
                        // All cards in this suit have been played already
                        // → winner's card is the best there was → not feeding
                    }
                }
            }

            // Add this trick's cards to the played set
            for card in &set.cards {
                played_so_far.push(*card);
            }

            leader = set.winner;
        }
    }

    /// Check if a player could have beaten the set winner with a different card.
    /// Now checks against actually playable cards, not just what was played.
    fn player_could_beat(
        &self,
        table: &Table,
        set: &game_core::Set,
        _player: usize,
        card_order_in_set: usize,
        winner: usize,
    ) -> bool {
        let winner_card = &set.cards[winner.min(set.cards.len() - 1)];
        let played_card = &set.cards[card_order_in_set];

        // If the card already beats the winner, they weren't feeding
        if card_beats(played_card, winner_card, table.trump_suit, Some(set.lead_suit)) {
            return true;
        }

        false
    }

    /// Probability that player a is the partner of player b (based on feeding observations).
    /// Returns a value 0.0..1.0. Higher = more likely partners.
    pub fn partner_probability(&self, a: usize, b: usize) -> f64 {
        if self.tricks_observed == 0 {
            return 0.0;
        }
        // If a feeds b, they're likely partners
        let feed_ab = self.feed_counts[a][b] as f64;
        let feed_ba = self.feed_counts[b][a] as f64;

        // Combined feed score normalized by tricks observed
        let total_feeds = feed_ab + feed_ba;
        let max_possible = self.tricks_observed as f64;

        if max_possible == 0.0 {
            return 0.0;
        }
        (total_feeds / max_possible).min(1.0)
    }

    /// Who I should aggressively compete against (the real threat).
    /// Returns the opponent I should prioritize beating: the bet winner if known,
    /// otherwise the opponent with the most tricks won.
    pub fn primary_threat(&self) -> Option<usize> {
        self.bet_winner
    }

    /// Who I should let have tricks (feed to).
    /// If I know my partner, feed to them. Otherwise, feed to whoever the
    /// observations suggest is most likely my partner.
    pub fn feed_target(&self) -> Option<usize> {
        if let Some(p) = self.my_partner {
            return Some(p);
        }
        // No known partner — infer from feeds
        let mut best_target = None;
        let mut best_score = 0f64;
        for other in 0..4 {
            if other == self.bot_idx {
                continue;
            }
            let prob = self.partner_probability(self.bot_idx, other);
            if prob > best_score {
                best_score = prob;
                best_target = Some(other);
            }
        }
        best_target
    }

    /// Is the player at `idx` on my team?
    pub fn is_teammate(&self, idx: usize) -> bool {
        idx == self.bot_idx || self.my_partner.map_or(false, |p| idx == p)
    }

    /// Is the player at `idx` the bet winner (primary threat)?
    pub fn is_bet_winner(&self, idx: usize) -> bool {
        self.bet_winner.map_or(false, |b| idx == b)
    }

    /// Estimate how much we want to beat this player's trick (0.0 = don't try, 1.0 = must beat).
    pub fn threat_level(&self, player_idx: usize) -> f64 {
        if self.is_teammate(player_idx) {
            return 0.0; // Never beat teammate
        }
        if self.is_bet_winner(player_idx) {
            return 1.0; // Always beat the bet winner
        }
        // The bet winner's partner: moderate threat
        0.5
    }
}

// ── Card play decision ─────────────────────────────────────────────────────

/// Decide which card to play during the playing phase (Easy difficulty).
fn decide_card_easy(table: &Table) -> Card {
    let legal = legal_plays(table);
    if legal.is_empty() {
        // Fallback: play the first card in hand
        return table.players[table.current_player]
            .hand
            .first()
            .copied()
            .unwrap_or(Card::new(Suit::Clubs, Rank::Two));
    }

    let legal_refs: Vec<&Card> = legal.iter().collect();

    if table.current_set_cards.is_empty() {
        // Leading: play strongest card
        let idx = strongest_card_index(&legal_refs, table.trump_suit, None);
        legal[idx]
    } else {
        // Following: try to win the trick
        let lead_suit = table.lead_suit;
        // Find current best card in the trick
        let current_best_idx = strongest_card_index(
            &table.current_set_cards.iter().collect::<Vec<&Card>>(),
            table.trump_suit,
            lead_suit,
        );
        let current_best = &table.current_set_cards[current_best_idx];

        // Try to find a legal card that beats the current best
        let winning_cards: Vec<&Card> = legal_refs
            .iter()
            .filter(|c| card_beats(c, current_best, table.trump_suit, lead_suit))
            .copied()
            .collect();

        if !winning_cards.is_empty() {
            // Play the weakest winning card
            let idx = weakest_card_index(&winning_cards, table.trump_suit, lead_suit);
            *winning_cards[idx]
        } else {
            // Can't win — play the weakest card
            let idx = weakest_card_index(&legal_refs, table.trump_suit, lead_suit);
            legal[idx]
        }
    }
}

/// Collect all cards already played (from completed sets and current set).
fn get_played_cards(table: &Table) -> Vec<Card> {
    let mut played = Vec::new();
    for set in &table.completed_sets {
        played.extend(set.cards.iter());
    }
    played.extend(table.current_set_cards.iter());
    played
}

/// For each suit, the highest rank that has NOT been played yet.
fn unplayed_strengths_by_suit(table: &Table) -> [u8; 4] {
    let played = get_played_cards(table);
    let mut per_suit = [14u8; 4]; // Ace=14 is highest

    for card in &played {
        let suit_idx = match card.suit {
            Suit::Clubs => 0,
            Suit::Diamonds => 1,
            Suit::Hearts => 2,
            Suit::Spades => 3,
        };
        let rank_val = card.rank as u8;
        // Only update if this exact rank was the current max (a simplification:
        // we track the highest remaining by noting which ranks are gone)
        if rank_val == per_suit[suit_idx] {
            // Step down to find the next highest
            for r in (2..rank_val).rev() {
                let still_available = !played.iter().any(|c| {
                    let si = match c.suit {
                        Suit::Clubs => 0,
                        Suit::Diamonds => 1,
                        Suit::Hearts => 2,
                        Suit::Spades => 3,
                    };
                    si == suit_idx && c.rank as u8 == r
                });
                if still_available {
                    per_suit[suit_idx] = r;
                    break;
                }
            }
            if per_suit[suit_idx] == rank_val {
                per_suit[suit_idx] = 0; // Nothing left in this suit
            }
        }
    }

    per_suit
}

/// Decide which card to play during the playing phase (Medium difficulty).
///
/// Team-aware strategy:
/// - Tracks which suits each player has played (void detection)
/// - If partner is winning the current trick: dumps weakest card (feed)
/// - If opponent winning and last to play: wins with minimal card
/// - If leading: prefers to lead a suit partner hasn't played (void-feed for trump),
///   otherwise plays strongest unplayed card
/// - Default: tries to win with strongest card, else dumps weakest
fn decide_card_medium(table: &Table) -> Card {
    let legal = legal_plays(table);
    if legal.is_empty() {
        return table.players[table.current_player]
            .hand
            .first()
            .copied()
            .unwrap_or(Card::new(Suit::Clubs, Rank::Two));
    }

    let legal_refs: Vec<&Card> = legal.iter().collect();
    let lead_suit = table.lead_suit;
    let trump = table.trump_suit;
    let partner = table.partner_idx;
    let num_cards = table.current_set_cards.len();

    // Build team model from observed play
    let mut team = TeamModel::new(0); // bot_idx will be set properly below
    // Find this bot's index by checking which player has the current bot's name
    // We approximate: the current player is us since the bot is being asked to play
    team.bot_idx = table.current_player;
    team.observe(table);

    // ── Leading ─────────────────────────────────────────────────────
    if table.current_set_cards.is_empty() {
        return decide_lead_medium(table, &legal, &legal_refs, partner);
    }

    // ── Following ───────────────────────────────────────────────────

    // Determine who is currently winning the trick
    let current_winner_idx = find_current_winner(&table.current_set_cards, trump, lead_suit);
    let leader = if num_cards == 0 {
        table.current_player
    } else {
        (table.current_player + 4 - num_cards) % 4
    };
    let current_winner_player = (leader + current_winner_idx) % 4;

    let partner_winning = partner.map_or(false, |p| current_winner_player == p);

    // Partner is winning: dump the weakest legal card (feed)
    if partner_winning {
        let idx = weakest_card_index(&legal_refs, trump, lead_suit);
        return legal[idx];
    }

    // Opponent winning: try to beat them with the weakest card that can win
    let current_best = &table.current_set_cards[current_winner_idx];
    let winning_cards: Vec<&Card> = legal_refs
        .iter()
        .filter(|c| card_beats(c, current_best, trump, lead_suit))
        .copied()
        .collect();

    if !winning_cards.is_empty() {
        // We can win — play the weakest winning card (conserves strength)
        let idx = weakest_card_index(&winning_cards, trump, lead_suit);
        return *winning_cards[idx];
    }

    // Can't win — dump the weakest legal card
    let idx = weakest_card_index(&legal_refs, trump, lead_suit);
    legal[idx]
}

/// Medium bot leading decision.
///
/// Strategy:
/// 1. If partner is known, prefer to lead a suit the partner has NOT played yet
///    (partner is void in that suit and can trump).
/// 2. If holding the strongest remaining card in a suit, lead it.
/// 3. Otherwise play the weakest led-legal card.
fn decide_lead_medium(
    table: &Table,
    legal: &[Card],
    legal_refs: &[&Card],
    partner: Option<usize>,
) -> Card {
    let trump = table.trump_suit;

    // Compute which suits each player has played so far (void tracking)
    let player_suits_played = compute_player_suits_played(table);

    // Strategy 1: Lead a suit partner hasn't played (void feed for trump)
    if let Some(partner_idx) = partner {
        // Check each legal lead — prefer a suit partner is void in
        // (hasn't played any card of that suit yet)
        let partner_void_suit = find_partner_void_suit(
            table, partner_idx, &player_suits_played, legal, trump
        );
        if let Some(partner_void_card) = partner_void_suit {
            return partner_void_card;
        }
    }

    // Strategy 2: Play strongest card from unplayed strengths
    let strengths = unplayed_strengths_by_suit(table);
    let mut card_strengths: Vec<(usize, u16)> = legal_refs
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let suit_idx = match c.suit {
                Suit::Clubs => 0,
                Suit::Diamonds => 1,
                Suit::Hearts => 2,
                Suit::Spades => 3,
            };
            let is_top = (c.rank as u8) >= strengths[suit_idx];
            let bonus = if is_top { 100u16 } else { 0u16 };
            (i, bonus + card_strength(c, trump, None))
        })
        .collect();

    card_strengths.sort_by(|a, b| b.1.cmp(&a.1));
    let best_idx = card_strengths[0].0;
    legal[best_idx]
}

/// Find which suits each player has played so far (from completed sets and current trick).
/// Returns a Vec of 4 Sets, one per player, containing the suits they've played.
fn compute_player_suits_played(table: &Table) -> [Vec<Suit>; 4] {
    let mut suits: [Vec<Suit>; 4] = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];

    // First set is led by (bet_winner + 1) % 4, subsequent by the previous set's winner
    let mut leader = (table.bet_winner.unwrap_or(0) + 1) % 4;

    for set in &table.completed_sets {
        for (i, card) in set.cards.iter().enumerate() {
            let player_idx = (leader + i) % 4;
            if !suits[player_idx].contains(&card.suit) {
                suits[player_idx].push(card.suit);
            }
        }
        // Next set is led by the winner of this set
        leader = set.winner;
    }

    // Also check the current trick
    let current_leader = if table.current_set_cards.is_empty() {
        table.current_player
    } else {
        (table.current_player + 4 - table.current_set_cards.len()) % 4
    };
    for (i, card) in table.current_set_cards.iter().enumerate() {
        let player_idx = (current_leader + i) % 4;
        if !suits[player_idx].contains(&card.suit) {
            suits[player_idx].push(card.suit);
        }
    }

    suits
}

/// If partner hasn't played a suit yet, they might be void there.
/// Try to find a legal card in a suit partner hasn't played.
/// Prefer non-trump suits first (to not waste partner's trump).
fn find_partner_void_suit(
    _table: &Table,
    partner_idx: usize,
    player_suits_played: &[Vec<Suit>; 4],
    legal: &[Card],
    trump: Option<Suit>,
) -> Option<Card> {
    let partner_played = &player_suits_played[partner_idx];
    let all_suits = [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades];

    // First pass: prefer non-trump suits partner is void in
    for &suit in &all_suits {
        if Some(suit) == trump {
            continue; // Skip trump in first pass
        }
        if partner_played.contains(&suit) {
            continue; // Partner has played this suit, not void
        }
        // This suit — partner hasn't played it, they might be void
        // Find our strongest card in this suit (from legal leads)
        let suit_cards: Vec<&Card> = legal.iter().filter(|c| c.suit == suit).collect();
        if !suit_cards.is_empty() {
            // Play our strongest card in this void suit
            let idx = strongest_card_index(&suit_cards, trump, None);
            return Some(*suit_cards[idx]);
        }
    }

    // Second pass: try trump suit partner is void in
    if let Some(tr) = trump {
        if !partner_played.contains(&tr) {
            let suit_cards: Vec<&Card> = legal.iter().filter(|c| c.suit == tr).collect();
            if !suit_cards.is_empty() {
                // Play our weakest trump — partner can over-trump if needed
                let idx = weakest_card_index(&suit_cards, trump, None);
                return Some(*suit_cards[idx]);
            }
        }
    }

    None
}

/// Find the index of the currently winning card in a trick (0-based within the trick).
fn find_current_winner(cards: &[Card], trump: Option<Suit>, lead_suit: Option<Suit>) -> usize {
    if cards.is_empty() {
        return 0;
    }
    let mut winner = 0usize;
    for i in 1..cards.len() {
        if card_beats(&cards[i], &cards[winner], trump, lead_suit) {
            winner = i;
        }
    }
    winner
}

/// Given cards played before a trick and the cards in the current trick,
/// find the strongest card in `suit` that hasn't been played yet.
/// Returns `None` if all cards in that suit have been played.
fn strongest_unplayed_in_suit(
    played_before: &[Card],
    current_trick: &[Card],
    suit: Suit,
) -> Option<Card> {
    // Check all ranks from Ace (14) down to Two (2)
    for rank_val in (2..=14u8).rev() {
        let rank = num_to_rank(rank_val)?;
        let candidate = Card::new(suit, rank);

        // Skip if it's been played before this trick
        if played_before.iter().any(|c| c.suit == suit && c.rank == rank) {
            continue;
        }
        // Skip if it's in the current trick
        if current_trick.iter().any(|c| c.suit == suit && c.rank == rank) {
            continue;
        }
        // Found the strongest unplayed card
        return Some(candidate);
    }
    None
}

/// Convert a numeric rank (2..=14) to a `Rank`. Returns None for invalid values.
fn num_to_rank(val: u8) -> Option<Rank> {
    match val {
        2 => Some(Rank::Two),
        3 => Some(Rank::Three),
        4 => Some(Rank::Four),
        5 => Some(Rank::Five),
        6 => Some(Rank::Six),
        7 => Some(Rank::Seven),
        8 => Some(Rank::Eight),
        9 => Some(Rank::Nine),
        10 => Some(Rank::Ten),
        11 => Some(Rank::Jack),
        12 => Some(Rank::Queen),
        13 => Some(Rank::King),
        14 => Some(Rank::Ace),
        _ => None,
    }
}

// ── Main entry point ───────────────────────────────────────────────────────

/// A decision made by the bot: what action to take.
/// The bot only recommends — actual game state mutation happens in the caller.
#[derive(Debug, Clone, Copy)]
pub enum BotAction {
    /// Place a bid (pass or bid)
    Call(Call),
    /// Select a partner card
    SelectPartner(Card),
    /// Play a card during the playing phase
    PlayCard(Card),
}

/// Read the current table state and automatically decide what action to take.
///
/// Returns a `BotAction` — the caller is responsible for applying it to the table.
/// This keeps the game state mutation logic out of the bot layer, ensuring the
/// bot can never accidentally perform illegal moves; the `Table` methods enforce
/// legality when the caller invokes them.
pub fn auto_decide(
    table: &Table,
    difficulty: BotDifficulty,
) -> Result<BotAction, &'static str> {
    match table.phase {
        GamePhase::Bidding => {
            let call = decide_bid(table);
            Ok(BotAction::Call(call))
        }
        GamePhase::PartnerSelection => {
            let card = decide_partner_card(table);
            Ok(BotAction::SelectPartner(card))
        }
        GamePhase::Playing => {
            let card = match difficulty {
                BotDifficulty::Easy => decide_card_easy(table),
                BotDifficulty::Medium => decide_card_medium(table),
            };
            Ok(BotAction::PlayCard(card))
        }
        _ => Err("No action defined for this phase"),
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use game_core::Card;

    #[test]
    fn test_card_beats_same_suit() {
        let ace = Card::new(Suit::Spades, Rank::Ace);
        let king = Card::new(Suit::Spades, Rank::King);
        assert!(card_beats(&ace, &king, None, None));
        assert!(!card_beats(&king, &ace, None, None));
    }

    #[test]
    fn test_card_beats_trump() {
        let trump = Card::new(Suit::Hearts, Rank::Two);
        let non_trump = Card::new(Suit::Spades, Rank::Ace);
        assert!(card_beats(&trump, &non_trump, Some(Suit::Hearts), None));
        assert!(!card_beats(&non_trump, &trump, Some(Suit::Hearts), None));
    }

    #[test]
    fn test_card_beats_led_suit() {
        let led = Card::new(Suit::Diamonds, Rank::Three);
        let off = Card::new(Suit::Clubs, Rank::Ace);
        assert!(card_beats(&led, &off, None, Some(Suit::Diamonds)));
    }

    #[test]
    fn test_legal_plays_follow_suit() {
        let mut table = Table::new(["N", "E", "S", "W"]);
        table.phase = GamePhase::Playing;
        table.current_player = 0;
        table.current_set_cards = vec![Card::new(Suit::Hearts, Rank::Ace)];
        table.lead_suit = Some(Suit::Hearts);
        // Give player 0 hearts and spades
        table.players[0] = game_core::Player::new("N");
        table.players[0].receive_card(Card::new(Suit::Hearts, Rank::King));
        table.players[0].receive_card(Card::new(Suit::Spades, Rank::Ace));

        let legal = legal_plays(&table);
        assert_eq!(legal.len(), 1);
        assert_eq!(legal[0].suit, Suit::Hearts);
        assert_eq!(legal[0].rank, Rank::King);
    }

    #[test]
    fn test_legal_plays_no_follow() {
        let mut table = Table::new(["N", "E", "S", "W"]);
        table.phase = GamePhase::Playing;
        table.current_player = 0;
        table.current_set_cards = vec![Card::new(Suit::Hearts, Rank::Ace)];
        table.lead_suit = Some(Suit::Hearts);
        // Give player 0 only spades (no hearts to follow)
        table.players[0] = game_core::Player::new("N");
        table.players[0].receive_card(Card::new(Suit::Spades, Rank::Ace));
        table.players[0].receive_card(Card::new(Suit::Spades, Rank::King));

        let legal = legal_plays(&table);
        assert_eq!(legal.len(), 2); // Can play any card
    }

    #[test]
    fn test_legal_plays_cannot_lead_trump() {
        let mut table = Table::new(["N", "E", "S", "W"]);
        table.phase = GamePhase::Playing;
        table.current_player = 0;
        table.current_set_cards = Vec::new(); // Leading
        table.trump_suit = Some(Suit::Hearts);
        table.trump_played = false;
        // Give player 0 hearts (trump) and spades
        table.players[0] = game_core::Player::new("N");
        table.players[0].receive_card(Card::new(Suit::Hearts, Rank::Ace));
        table.players[0].receive_card(Card::new(Suit::Spades, Rank::King));

        let legal = legal_plays(&table);
        assert_eq!(legal.len(), 1);
        assert_eq!(legal[0].suit, Suit::Spades);
    }

    #[test]
    fn test_suit_scoring() {
        let hand = vec![
            Card::new(Suit::Spades, Rank::Ace),
            Card::new(Suit::Spades, Rank::King),
            Card::new(Suit::Spades, Rank::Queen),
            Card::new(Suit::Hearts, Rank::Two),
        ];
        // Spades: 3 cards * 2 = 6, picture values: A=4, K=3, Q=2 => 9, total = 15
        let score = suit_score(&hand, Suit::Spades);
        assert_eq!(score, 15);
        // Hearts: 1 card * 2 = 2, picture values: 0, total = 2
        let score_h = suit_score(&hand, Suit::Hearts);
        assert_eq!(score_h, 2);
    }

    #[test]
    fn test_decide_bid() {
        let mut table = Table::new(["N", "E", "S", "W"]);
        table.players[0] = game_core::Player::new("N");
        // Give North a strong spade hand
        for rank in [
            Rank::Ace, Rank::King, Rank::Queen, Rank::Jack,
            Rank::Ten, Rank::Nine, Rank::Eight,
        ] {
            table.players[0].receive_card(Card::new(Suit::Spades, rank));
        }
        table.phase = GamePhase::Bidding;
        let _call = decide_bid(&table);
        // Should bid something (spades, level 3 since score is high)
    }
}

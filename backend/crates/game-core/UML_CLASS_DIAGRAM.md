# Game Core UML Class Diagram

This diagram is derived from the Rust domain model in `backend/crates/game-core/src`.
It focuses on the core entities, enums, and their relationships in the bridge game engine.

```mermaid
classDiagram

class Suit {
  <<enumeration>>
  Clubs
  Diamonds
  Hearts
  Spades
}

class Rank {
  <<enumeration>>
  Two
  Three
  Four
  Five
  Six
  Seven
  Eight
  Nine
  Ten
  Jack
  Queen
  King
  Ace
}

class Card {
  +suit: Suit
  +rank: Rank
  +new(suit, rank) Card
  +to_unicode_string() String
  +to_ascii_string() String
}

class Strain {
  <<enumeration>>
  Clubs
  Diamonds
  Hearts
  Spades
}

class Bid {
  +level: u8
  +strain: Strain
  +new(level, strain) Bid
  +parse(s) Option~Bid~
}

class Call {
  <<enumeration>>
  Bid(Bid)
  Pass
}

class Contract {
  +bid: Bid
  +declarer: usize
  +tricks_required() u8
}

class AuctionState {
  +current_player: usize
  +last_bid: Option~Bid~
  +last_bidder: Option~usize~
  +consecutive_passes: u8
  +call_history: Vec~Call~
  +new(dealer) AuctionState
  +make_call(call) Result
  +is_ended() bool
  +final_contract() Option~Contract~
}

class Deck {
  -cards: Vec~Card~
  +new() Deck
  +shuffle()
  +draw() Card
  +size() usize
  +is_empty() bool
}

class Direction {
  <<enumeration>>
  North
  East
  South
  West
}

class Player {
  +name: String
  +hand: Vec~Card~
  +new(name) Player
  +receive_card(card)
  +hand_size() usize
  +sort_hand()
  +play_card(index) Card
  +has_card(card) bool
  +hand_string() String
}

class Vulnerability {
  <<enumeration>>
  None
  NorthSouth
  EastWest
  Both
}

class Set {
  +cards: [Card;4]
  +winner: usize
  +lead_suit: Suit
  +new(cards, trump) Set
}

class ContractResult {
  <<enumeration>>
  Made(u8)
  Down(u8)
}

class DealScore {
  +contract: Contract
  +result: ContractResult
  +declarer_points: i32
  +defender_points: i32
}

class GamePhase {
  <<enumeration>>
  Dealing
  Bidding
  PartnerSelection
  Playing
  Scoring
  Finished
}

class Table {
  +players: [Player;4]
  +vulnerability: Vulnerability
  +dealer: Direction
  +phase: GamePhase
  +deck: Deck
  +auction: Option~AuctionState~
  +bet_size: u8
  +trump_suit: Option~Suit~
  +bet_winner: Option~usize~
  +partner_idx: Option~usize~
  +partner_card: Option~Card~
  +current_set_cards: Vec~Card~
  +completed_sets: Vec~Set~
  +sets_won: [u8;4]
  +lead_suit: Option~Suit~
  +trump_played: bool
  +current_player: usize
  +contract: Option~Contract~
  +score: Option~DealScore~
  +new(player_names) Table
  +deal()
  +make_call(call) Result
  +select_partner(card) Result
  +play_card(card) Result
  +current_player_index() usize
  +next_deal()
}

Card --> Suit
Card --> Rank
Bid --> Strain
Call --> Bid
Contract --> Bid
AuctionState --> Bid
AuctionState --> Call
AuctionState --> Contract
Strain ..> Suit : from_suit()

Deck *-- "0..52" Card : cards
Player *-- "0..13" Card : hand
Set *-- "4" Card : cards
Set --> Suit : lead_suit

DealScore --> Contract
DealScore --> ContractResult

Table *-- "4" Player : players
Table *-- Deck : deck
Table o-- "0..1" AuctionState : auction
Table o-- "0..1" Contract : contract
Table o-- "0..1" DealScore : score
Table *-- "0..*" Set : completed_sets
Table *-- "0..4" Card : current_set_cards
Table --> Vulnerability
Table --> Direction
Table --> GamePhase
Table --> Suit : trump_suit / lead_suit
Table --> Card : partner_card
```

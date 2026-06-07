/** @file Game type definitions for Singapore Bridge */

/**
 * Full game state.
 * Tracks players, teams, phase, current trick, and win conditions.
 */
export interface Game {
    /** The 4 players (index 0 = P1/human, 1 = P2, 2 = P3, 3 = P4) */
    Players: Player[]
    /** Bet winner + their partner (set after partner selection) */
    Team1: Player[]
    /** Player who won the betting phase */
    BetWinner: Player
    /** The card whose holder becomes the bet winner's partner */
    PartnerCard: Card
    /** Opposing team (set after partner selection) */
    Team2: Player[]
    /** Trump suit for this round */
    Trump: string
    /** Number of tricks the bet winner's team must take above 6 */
    BetSize: number
    /** True during betting phase */
    IsBettingPhase: boolean
    /** True during partner selection phase (between betting and play) */
    IsPartnerSelectionPhase: boolean
    /** Whether trump has been played in any trick (can't lead trump until true) */
    TrumpPlayed: boolean
    /** All 52 shuffled cards (used for partner selection UI) */
    FullDeck: Card[]
    /** Current trick cards being played (or betting history during betting phase) */
    Moves: Move[]
    /** Previously completed trick (for display) */
    PreviousMoves: Move[]
    /** ID of the player whose turn it is (1-4) */
    WhoseTurn: number
    /** Suit led in the current trick (empty if not set yet) */
    TurnSuit: string
    /** "Team 1" or "Team 2" when someone wins, "" during play */
    Winner: string
    /** Whether bot auto-play is enabled */
    TurnOnBots: boolean
    /** All completed sets (tricks) during the playing phase */
    CompletedSets: CompletedSet[]
}

/** A single playing card */
export interface Card {
    /** Display rank: "2"–"10", "J", "Q", "K", "A" */
    Rank: string
    /** Numeric value: 2–14 (Ace = 14) */
    Value: number
    /** Suit name: "Club", "Diamond", "Heart", "Spades" */
    Suit: string
    /** Whether this card won a set (shown with crown icon) */
    WonSet: boolean
}

/** A player at the table */
export interface Player {
    /** 1 = human (you), 2–4 = bots */
    ID: number
    /** Cards currently held in hand */
    Cards: Card[]
    /** Cards this player has played this round */
    PlayedCards: Card[]
    /** This player's partner (null until partner selection) */
    Partner: Player | null
    /** Number of sets (tricks) won this round */
    Sets: number
    /** Whether this player is controlled by AI */
    IsBot: boolean
    /** Display name */
    Username: string
    /** One-character abbreviation */
    ShortUsername: string
}

/** A single action in a turn (card played by a player) */
export interface Move {
    /** The card that was played */
    CardPlayed: Card
    /** Which player played it */
    PlayerID: number
}

/** A completed set (trick) — all 4 cards played and who won */
export interface CompletedSet {
    /** The 4 cards played (index 0 = lead) */
    Cards: Card[]
    /** Player ID (1-4) who won this set */
    WinnerID: number
    /** Player ID (1-4) who played each card (same length as Cards) */
    PlayerIDs: number[]
}

/** Database record for a completed match */
export type MatchRecord = {
    id: number
    userID: number
    date: number
    botDifficulty: string
    trumpSuit: string
    betSize: number
    betWinner: number
    partner: number | null
    wonMatch: number | null
    player1Sets: number
    player2Sets: number
    player3Sets: number
    player4Sets: number
    player1Hand: string
    player2Hand: string
    player3Hand: string
    player4Hand: string
    /** JSON string of completed sets: [{Cards: Card[], WinnerID: number}] */
    setsData: string | null
    /** Compact hand preview strings for quick display */
    preview1: string | null
    preview2: string | null
    preview3: string | null
    preview4: string | null
    /** Whether this match is hidden (affects leaderboard/elo) */
    isHidden?: boolean
}

export interface PlayEvent {
    /** deterministic key (e.g. `${trickIndex}-${position}-${playerId}-${card.Suit}${card.Value}`) */
    id: string           
    trickIndex: number
    /** 0..3 within trick */
    position: number      
    playerId: number
    card: Card
    isTrickEnd: boolean
    trickWinnerId?: number | null
}

export const EMPTY_PLAYER: Player = {
        ID: 0,
        Cards: [],
        PlayedCards: [],
        Partner: null,
        Sets: 0,
        IsBot: false,
        Username: "",
        ShortUsername: "",
}

export const EMPTY_GAME: Game = {
	Players: [],
	Team1: [],
	BetWinner: EMPTY_PLAYER,
	PartnerCard: { Rank: "", Value: 0, Suit: "", WonSet: false },
	Team2: [],
	Trump: "Club",
	BetSize: 0,
	IsBettingPhase: false,
	IsPartnerSelectionPhase: false,
	TrumpPlayed: false,
	FullDeck: [],
	Moves: [],
	PreviousMoves: [],
	WhoseTurn: 1,
	TurnSuit: "",
	Winner: "",
	TurnOnBots: false,
	CompletedSets: [],
}
// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces
declare global {
	namespace App {
		// interface Error {}
		// interface Locals {}
		// interface PageData {}
		// interface PageState {}
		// interface Platform {}
	}
}

// ── Game type definitions ──────────────────────────────────────────
// These are globally available (no import needed) for .svelte files.
// Game-logic .ts files should import from "$lib/game/types" instead.
//
// When adding/updating a type, update BOTH this file AND src/lib/game/types.ts.

interface Game {
	Players: Player[]
	Team1: Player[]
	BetWinner: Player
	PartnerCard: Card
	Team2: Player[]
	Trump: string
	BetSize: number
	IsBettingPhase: boolean
	IsPartnerSelectionPhase: boolean
	TrumpPlayed: boolean
	FullDeck: Card[]
	Moves: Move[]
	PreviousMoves: Move[]
	WhoseTurn: number
	TurnSuit: string
	Winner: string
	TurnOnBots: boolean
	CompletedSets: CompletedSet[]
}

interface Card {
	Rank: string
	Value: number
	Suit: string
	WonSet: boolean
}

interface Player {
	ID: number
	Cards: Card[]
	PlayedCards: Card[]
	Partner: Player | null
	Sets: number
	IsBot: boolean
	Username: string
	ShortUsername: string
}

interface Move {
	CardPlayed: Card
	PlayerID: number
}

type MatchRecord = {
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
    setsData: string | null
    preview1: string | null
    preview2: string | null
    preview3: string | null
    preview4: string | null
    isHidden?: boolean
}

interface PlayEvent {
	kind: "play"
	id: string           
	trickIndex: number
	position: number      
	playerId: number
	card: Card
	isTrickEnd: boolean
	trickWinnerId?: number | null
}

interface WinEvent { 
	kind: "win"
	id: string
	winner: string 
}

type GameEvent = PlayEvent | WinEvent

interface CompletedSet {
	Cards: Card[]
	WinnerID: number
	PlayerIDs: number[]
}

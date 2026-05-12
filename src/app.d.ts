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

interface Game {
	Players: Player[]
	Team1: Player[] // winner of bet
	BetWinner: Player
	PartnerCard: Card
	Team2: Player[]
	Trump: string // trump suit
	BetSize: number
	IsBettingPhase: boolean
	Moves: Move[]
	WhoseTurn: number
	TrumpPlayed: boolean
	CardsPlayed: Card[]
	TurnSuit: string
	Winner: string
	TurnOnBots: boolean
}

interface Card {
	Rank: string // 2 to Ace
	Value: number // 2-14
	Suit: string
	WonSet: boolean
}

interface Player {
	ID: number // 1-4
	Cards: Card[]
	PlayedCards: Card[]
	Partner: Player | null
	Sets: number
}

interface Move {
	CardPlayed: Card
	PlayerID: number
}
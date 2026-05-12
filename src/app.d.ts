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
	Team2: Player[]
	Trump: string // trump suit
	BetSize: number
	IsBettingPhase: boolean
	Moves: Move[]
	WhoseTurn: number
	TrumpPlayed: boolean
	TurnSuit: string
	Winner: string
}

interface Card {
	Rank: string // 2 to Ace
	Value: number // 2-14
	Suit: string
}

interface Player {
	ID: number // 1-4
	Cards: Card[]
	Sets: number
}

interface Move {
	CardPlayed: Card
	PlayerID: number
}
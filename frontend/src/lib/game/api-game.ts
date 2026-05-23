/** @file API adapter for playing games against the Rust backend.
 *  Maps between the Rust backend's JSON state and the frontend's Game type. */

import type { Game, Card, Player, Move } from "./types"
import { SUIT_ENUM, VALUE_TO_RANK, SUIT_SORT_ORDER } from "./cards"

const API_URL = "http://127.0.0.1:3000"

// ── API Type Definitions ───────────────────────────────────────────

interface ApiState {
    phase: string
    hands: string[]
    currentPlayer: number
    betSize: number
    trumpSuit: string
    betWinner: number | null
    partnerIdx: number | null
    setsWon: number[]
    completedSetCount: number
    isFinished: boolean
    currentTrickCards: Array<{ suit: string; rank: string }>
    currentTrickStartPlayer: number
    previousTrickCards: Array<{ suit: string; rank: string }>
    previousTrickWinner: number | null
}

interface ApiNewGameResponse {
    ok: boolean
    roomId?: string
    state?: ApiState
}

interface ApiActionResponse {
    ok: boolean
    state?: ApiState
}

// ── Suit/Rank Mapping ──────────────────────────────────────────────

/** API uses plural suit names; frontend uses singular. */
const API_SUIT_TO_FRONTEND: Record<string, string> = {
    Clubs: "Club",
    Diamonds: "Diamond",
    Hearts: "Heart",
    Spades: "Spades",
}

const FRONTEND_SUIT_TO_API: Record<string, string> = {
    Club: "Clubs",
    Diamond: "Diamonds",
    Heart: "Hearts",
    Spades: "Spades",
}

/** Map from short rank char(s) used in hand strings to numeric value. */
const SHORT_RANK_TO_VALUE: Record<string, number> = {
    A: 14, K: 13, Q: 12, J: 11,
    "10": 10, "9": 9, "8": 8, "7": 7,
    "6": 6, "5": 5, "4": 4, "3": 3, "2": 2,
}

/** Map from short suit char to frontend suit name. */
const SHORT_SUIT_TO_SUIT: Record<string, string> = {
    S: "Spades",
    H: "Heart",
    D: "Diamond",
    C: "Club",
}

/** Map from numeric value to short rank string for serialization. */
const VALUE_TO_SHORT_RANK: Record<number, string> = {
    14: "A", 13: "K", 12: "Q", 11: "J",
    10: "10", 9: "9", 8: "8", 7: "7",
    6: "6", 5: "5", 4: "4", 3: "3", 2: "2",
}

/** Map from API rank name (e.g. "Two", "Ace") to numeric value. */
const API_RANK_TO_VALUE: Record<string, number> = {
    Two: 2, Three: 3, Four: 4, Five: 5,
    Six: 6, Seven: 7, Eight: 8, Nine: 9,
    Ten: 10, Jack: 11, Queen: 12, King: 13, Ace: 14,
}

const VALUE_TO_API_RANK: Record<number, string> = {
    2: "Two", 3: "Three", 4: "Four", 5: "Five",
    6: "Six", 7: "Seven", 8: "Eight", 9: "Nine",
    10: "Ten", 11: "Jack", 12: "Queen", 13: "King", 14: "Ace",
}

// ── Hand String Parsing ───────────────────────────────────────────

/**
 * Parse a hand string like "AS KH 3C 10D" into Card[].
 * Each token: rank part (1-2 chars) + suit initial (1 char).
 */
function parseHandString(handStr: string): Card[] {
    const tokens = handStr.trim().split(/\s+/)
    const cards: Card[] = []
    for (const token of tokens) {
        if (!token) continue
        const rankStr = token.slice(0, -1)  // everything but last char
        const suitChar = token.slice(-1)     // last char
        const value = SHORT_RANK_TO_VALUE[rankStr]
        const suit = SHORT_SUIT_TO_SUIT[suitChar]
        if (value === undefined || !suit) {
            console.warn(`api-game: skipping unparseable card token "${token}"`)
            continue
        }
        cards.push({
            Rank: VALUE_TO_RANK[value] ?? String(value),
            Value: value,
            Suit: suit,
            WonSet: false,
        })
    }
    // Sort by suit order then value (matching sortHands in deck.ts)
    cards.sort((a, b) => {
        const suitDiff = (SUIT_SORT_ORDER[a.Suit] ?? 0) - (SUIT_SORT_ORDER[b.Suit] ?? 0)
        return suitDiff !== 0 ? suitDiff : a.Value - b.Value
    })
    return cards
}

// ── State Mapping ──────────────────────────────────────────────────

/**
 * Converts the backend's API state into the frontend Game type.
 * The Game object is the data structure the Svelte UI expects.
 */
export function apiStateToGame(state: ApiState, roomId: string, betWinnerIdx?: number): Game {
    const hands = state.hands.map(parseHandString)

    // Build players
    const players: Player[] = hands.map((hand, i) => ({
        ID: i + 1,
        Cards: hand,
        PlayedCards: [],
        Partner: null,
        Sets: state.setsWon[i] ?? 0,
        IsBot: i !== 0,
        Username: i === 0 ? "You" : `Bot ${i + 1}`,
        ShortUsername: i === 0 ? "Y" : `B${i + 1}`,
    }))

    // Map API suit to frontend suit
    const trump = state.trumpSuit
        ? (API_SUIT_TO_FRONTEND[state.trumpSuit] ?? state.trumpSuit)
        : "Club"

    const phase = state.phase
    const isBetting = phase === "Bidding"
    const isPartnerSelection = phase === "PartnerSelection"
    const isFinished = phase === "Finished" || state.isFinished

    // Determine winner
    let winner = ""
    if (isFinished) {
        // We can't know which team won from state alone — infer from sets
        // Team 1 (bet winner's team) needs 6+betSize sets
        if (betWinnerIdx !== undefined && betWinnerIdx >= 0) {
            const bw = betWinnerIdx
            const partner = state.partnerIdx ?? -1
            const team1Total = state.setsWon.filter((_, i) => i === bw || i === partner).reduce((a, b) => a + b, 0)
            const target = 6 + state.betSize
            winner = team1Total >= target ? "Team 1" : "Team 2"
        } else {
            winner = "Team 1" // fallback
        }
    }

    const suitOrder: Record<string, number> = { Spades: 0, Heart: 1, Club: 2, Diamond: 3 }

    // Build moves from current trick cards
    const moves: Move[] = (state.currentTrickCards || []).map((apiCard, i) => {
        const playerIdx = ((state.currentTrickStartPlayer ?? 0) + i) % 4
        const value = API_RANK_TO_VALUE[apiCard.rank] ?? 2
        return {
            CardPlayed: {
                Rank: VALUE_TO_RANK[value] ?? String(value),
                Value: value,
                Suit: API_SUIT_TO_FRONTEND[apiCard.suit] ?? apiCard.suit,
                WonSet: false,
            },
            PlayerID: playerIdx + 1,
        }
    })

    // Build previous moves from completed trick cards
    const prevMoves: Move[] = (state.previousTrickCards || []).map((apiCard, i) => {
        const value = API_RANK_TO_VALUE[apiCard.rank] ?? 2
        return {
            CardPlayed: {
                Rank: VALUE_TO_RANK[value] ?? String(value),
                Value: value,
                Suit: API_SUIT_TO_FRONTEND[apiCard.suit] ?? apiCard.suit,
                WonSet: false,
            },
            PlayerID: i + 1, // placeholder — real player mapping needs leader
        }
    })

    // WhoseTurn: backend is 0-indexed, frontend is 1-indexed
    const whoseTurn = state.currentPlayer + 1

    // Build a full deck for partner selection (52 unique cards)
    const fullDeck = buildFullDeck(hands)

    const game: Game = {
        Players: players,
        Team1: [],
        BetWinner: players[0],
        PartnerCard: fullDeck[0],
        Team2: [],
        Trump: trump,
        BetSize: state.betSize,
        IsBettingPhase: isBetting,
        IsPartnerSelectionPhase: isPartnerSelection,
        TrumpPlayed: false, // API doesn't expose this
        FullDeck: fullDeck,
        Moves: moves,
        PreviousMoves: prevMoves,
        WhoseTurn: whoseTurn,
        TurnSuit: "",
        Winner: winner,
        TurnOnBots: false, // backend handles bots
    }

    // Set bet winner and partner if available
    if (betWinnerIdx !== undefined && betWinnerIdx >= 0 && betWinnerIdx < 4) {
        game.BetWinner = players[betWinnerIdx]
    }
    if (state.partnerIdx !== null && state.partnerIdx !== undefined && state.partnerIdx >= 0) {
        game.Players[state.partnerIdx].Partner = players[betWinnerIdx ?? 0]
        players[betWinnerIdx ?? 0].Partner = players[state.partnerIdx]
        // Link opponents
        const linked = new Set([betWinnerIdx ?? 0, state.partnerIdx])
        const opponents = players.filter((_, i) => !linked.has(i))
        if (opponents.length === 2) {
            opponents[0].Partner = opponents[1]
            opponents[1].Partner = opponents[0]
        }
        game.Team1 = [players[betWinnerIdx ?? 0], players[state.partnerIdx]]
        game.Team2 = players.filter((_, i) => !linked.has(i))
    }

    // PartnerCard: try to find a representative card from the partner's hand
    if (state.partnerIdx !== null && state.partnerIdx !== undefined && state.partnerIdx >= 0) {
        const partnerHand = players[state.partnerIdx].Cards
        if (partnerHand.length > 0) {
            game.PartnerCard = partnerHand[Math.floor(partnerHand.length / 2)]
        }
    }

    if (state.betSize > 0) {
        game.BetSize = state.betSize
    }

    return game
}

/**
 * Builds a full 52-card deck that excludes cards already in players' hands.
 * Used for the partner selection UI grid.
 */
function buildFullDeck(hands: Card[][]): Card[] {
    const inHands = new Set<string>()
    for (const hand of hands) {
        for (const card of hand) {
            inHands.add(`${card.Suit}:${card.Value}`)
        }
    }

    const suits = ["Club", "Diamond", "Heart", "Spades"]
    const deck: Card[] = []
    for (const suit of suits) {
        for (let val = 2; val <= 14; val++) {
            if (!inHands.has(`${suit}:${val}`)) {
                deck.push({
                    Rank: VALUE_TO_RANK[val],
                    Value: val,
                    Suit: suit,
                    WonSet: false,
                })
            }
        }
    }
    // Sort like sortHands
    deck.sort((a, b) => {
        const suitDiff = (SUIT_SORT_ORDER[a.Suit] ?? 0) - (SUIT_SORT_ORDER[b.Suit] ?? 0)
        return suitDiff !== 0 ? suitDiff : a.Value - b.Value
    })
    return deck
}

// ── API Calls ──────────────────────────────────────────────────────

/**
 * Creates a new single-player online game via the Rust backend.
 *
 * @param username - Display name for the human player
 * @param difficulty - "Easy" or "Medium"
 * @param token - Session token for X-Session-Token header
 * @returns The room ID and mapped Game state
 */
export async function createOnlineGame(
    username: string,
    difficulty: string,
    token: string,
): Promise<{ roomId: string; game: Game }> {
    const res = await fetch(`${API_URL}/api/game/new`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
            "X-Session-Token": token,
        },
        body: JSON.stringify({ difficulty }),
    })

    if (!res.ok) {
        const text = await res.text().catch(() => "")
        throw new Error(`Failed to create game: ${res.status} ${res.statusText}${text ? ` — ${text}` : ""}`)
    }

    const data: ApiNewGameResponse = await res.json()
    if (!data.ok || !data.roomId || !data.state) {
        throw new Error(`API returned error: ${JSON.stringify(data)}`)
    }

    const game = apiStateToGame(data.state, data.roomId)
    // Set human username
    game.Players[0].Username = username || "You"
    game.Players[0].ShortUsername = username ? username[0] : "Y"

    return { roomId: data.roomId, game }
}

/**
 * Fetches the current state of a game room.
 */
export async function getRoomState(roomId: string, token: string): Promise<Game> {
    const res = await fetch(`${API_URL}/room/${roomId}/state`, {
        headers: {
            "X-Session-Token": token,
        },
    })

    if (!res.ok) {
        throw new Error(`Failed to fetch room state: ${res.status}`)
    }

    const data = await res.json()
    const state: ApiState = data.state ?? data
    return apiStateToGame(state, roomId)
}

// ── Action Helpers ─────────────────────────────────────────────────

/** Call value types for the API action endpoint. */
type ApiCall = "Pass" | { Bid: { level: number; strain: string } } | { Double: Record<string, never> } | { Redouble: Record<string, never> }

/** API card format. */
interface ApiCard {
    suit: string
    rank: string
}

/**
 * Sends a bid action to the backend.
 *
 * @param roomId - The game room ID
 * @param token - Session token
 * @param call - "Pass" or a bid object with level and strain
 * @returns Updated Game state
 */
export async function doBid(roomId: string, token: string, call: ApiCall): Promise<Game> {
    return doAction(roomId, token, { type: "bid", call })
}

/**
 * Sends a play action to the backend.
 *
 * @param roomId - The game room ID
 * @param token - Session token
 * @param card - The card to play (frontend Card type)
 * @returns Updated Game state
 */
export async function doPlay(roomId: string, token: string, card: Card): Promise<Game> {
    const apiCard: ApiCard = {
        suit: FRONTEND_SUIT_TO_API[card.Suit] ?? card.Suit,
        rank: VALUE_TO_API_RANK[card.Value] ?? "Two",
    }
    return doAction(roomId, token, { type: "play", card: apiCard })
}

/**
 * Sends a partner selection action to the backend.
 *
 * @param roomId - The game room ID
 * @param token - Session token
 * @param card - The chosen partner card
 * @returns Updated Game state
 */
export async function doSelectPartner(roomId: string, token: string, card: Card): Promise<Game> {
    const apiCard: ApiCard = {
        suit: FRONTEND_SUIT_TO_API[card.Suit] ?? card.Suit,
        rank: VALUE_TO_API_RANK[card.Value] ?? "Two",
    }
    return doAction(roomId, token, { type: "selectPartner", card: apiCard })
}

// ── Core Action ────────────────────────────────────────────────────

interface ActionPayload {
    type: "bid" | "play" | "selectPartner"
    call?: ApiCall
    card?: ApiCard
}

async function doAction(roomId: string, token: string, payload: ActionPayload): Promise<Game> {
    const body: Record<string, unknown> = { type: payload.type }
    if (payload.call !== undefined) body.call = payload.call
    if (payload.card !== undefined) body.card = payload.card

    const res = await fetch(`${API_URL}/api/game/${roomId}/action`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
            "X-Session-Token": token,
        },
        body: JSON.stringify(body),
    })

    if (!res.ok) {
        const text = await res.text().catch(() => "")
        throw new Error(`Action failed: ${res.status} ${res.statusText}${text ? ` — ${text}` : ""}`)
    }

    const data: ApiActionResponse = await res.json()
    if (!data.ok || !data.state) {
        throw new Error(`API action returned error: ${JSON.stringify(data)}`)
    }

    // Extract betWinner from state for mapping
    const betWinnerIdx = data.state.betWinner ?? undefined

    return apiStateToGame(data.state, roomId, betWinnerIdx)
}

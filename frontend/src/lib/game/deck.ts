/** @file Deck creation, shuffling, dealing, and game initialization */

import type { Card, Game, Player } from "./types"
import { SUIT_ENUM, VALUE_TO_RANK, SUIT_SORT_ORDER } from "./cards"

/**
 * Generates a standard 52-card deck (no jokers), shuffles it,
 * deals 13 cards to each of 4 players, and returns a fresh Game state.
 *
 * Player 1 is the human; players 2–4 are bots.
 *
 * @param username - Display name for the human player
 */
export function initGame(username: string): Game {
    const deck = createDeck()
    shuffle(deck)
    const hands = deal(deck)
    sortHands(hands)

    const players = createPlayers(username, hands)

    return {
        Players: players,
        Team1: [],
        BetWinner: players[0],
        PartnerCard: deck[0],
        Team2: [],
        Trump: "Club",
        BetSize: 0,
        IsBettingPhase: true,
        IsPartnerSelectionPhase: false,
        TrumpPlayed: false,
        FullDeck: deck,
        Moves: [],
        PreviousMoves: [],
        WhoseTurn: 1,
        TurnSuit: "",
        Winner: "",
        TurnOnBots: true,
    }
}

// ── Deck Helpers ───────────────────────────────────────────────────

/**
 * Creates all 52 cards: 4 suits × 13 values (2–14).
 * Order: Club 2–14, Diamond 2–14, Heart 2–14, Spades 2–14.
 */
function createDeck(): Card[] {
    const deck: Card[] = []
    for (let suitIdx = 0; suitIdx < 4; suitIdx++) {
        for (let val = 2; val <= 14; val++) {
            deck.push({
                Rank: VALUE_TO_RANK[val],
                Value: val,
                Suit: SUIT_ENUM[suitIdx],
                WonSet: false,
            })
        }
    }
    return deck
}

/** Fisher-Yates shuffle (in-place). */
function shuffle(deck: Card[]): void {
    for (let i = deck.length - 1; i > 0; i--) {
        const j = Math.floor(Math.random() * (i + 1))
        ;[deck[i], deck[j]] = [deck[j], deck[i]]
    }
}

/** Splits the deck into 4 hands of 13 cards each (deck untouched via slice). */
function deal(deck: Card[]): Card[][] {
    const hands: Card[][] = []
    for (let i = 0; i < 4; i++) {
        hands.push(deck.slice(i * 13, (i + 1) * 13))
    }
    return hands
}

/** Sorts each hand by suit (Spades first), then value ascending. */
function sortHands(hands: Card[][]): void {
    for (const hand of hands) {
        hand.sort((a, b) => {
            const suitDiff = (SUIT_SORT_ORDER[a.Suit] ?? 0) - (SUIT_SORT_ORDER[b.Suit] ?? 0)
            return suitDiff !== 0 ? suitDiff : a.Value - b.Value
        })
    }
}

/** Creates the 4 player objects from the dealt hands. */
function createPlayers(username: string, hands: Card[][]): Player[] {
    return hands.map((hand, i) => {
        const id = i + 1
        return {
            ID: id,
            Cards: hand,
            PlayedCards: [],
            Partner: null,
            Sets: 0,
            IsBot: id !== 1,
            Username: id === 1 && username !== "" ? username : `Player ${id}`,
            ShortUsername: id === 1 && username !== "" ? username[0] : `P${id}`,
        }
    })
}

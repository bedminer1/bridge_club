/** @file Card comparison and suit/rank utility functions */

import type { Card, Game } from "./types"

// ── Suit & Rank Maps ───────────────────────────────────────────────

/** Maps numeric suit index to suit name (matching card generation order). */
export const SUIT_ENUM: Record<number, string> = {
    0: "Club",
    1: "Diamond",
    2: "Heart",
    3: "Spades",
}

/** Maps numeric card value to display rank string. */
export const VALUE_TO_RANK: Record<number, string> = {
    2: "2", 3: "3", 4: "4", 5: "5", 6: "6", 7: "7", 8: "8",
    9: "9", 10: "10", 11: "J", 12: "Q", 13: "K", 14: "A",
}

/**
 * Suit display order for sorting cards in hand.
 * Spades first (highest), then Hearts, Clubs, Diamonds.
 */
export const SUIT_SORT_ORDER: Record<string, number> = {
    Club: 0,
    Diamond: 1,
    Heart: 2,
    Spades: 3,
}

/**
 * Suit priority for betting (higher = more valuable).
 * Used in isLegalRaise to compare equal-sized bets.
 */
export const SUIT_BET_PRIORITY: Record<string, number> = {
    Club: 0,
    Diamond: 1,
    Heart: 2,
    Spades: 3,
}

// ── Card Comparison ────────────────────────────────────────────────

/**
 * Returns true if `c1` beats `c2` in the current game context.
 *
 * Priority order:
 * 1. Same suit → higher value wins
 * 2. Trump beats non-trump
 * 3. Led suit (TurnSuit) beats off-suit non-trump
 * 4. Otherwise, c1 does not beat c2
 *
 * @param game - Current game state (provides Trump + TurnSuit)
 * @param c1   - The candidate "beater" card
 * @param c2   - The card to compare against
 */
export function doesCard1Beat(game: Game, c1: Card, c2: Card): boolean {
    if (c1.Suit === c2.Suit) return c1.Value > c2.Value
    if (c1.Suit === game.Trump && c2.Suit !== game.Trump) return true
    if (c1.Suit !== game.Trump && c2.Suit === game.Trump) return false
    if (c1.Suit === game.TurnSuit && c2.Suit !== game.TurnSuit) return true
    return false
}

/**
 * Finds the strongest card in a list using `doesCard1Beat`.
 * The first card is used as the initial candidate.
 *
 * @param game  - Current game state
 * @param cards - Cards to evaluate (must be non-empty)
 */
export function findStrongestCard(game: Game, cards: Card[]): Card {
    let strongest = cards[0]
    for (const card of cards) {
        if (doesCard1Beat(game, card, strongest)) {
            strongest = card
        }
    }
    return strongest
}

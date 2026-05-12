/** @file Betting-phase logic: raise, pass, and transition to partner selection */

import { autoSelectPartner } from "./bot"
import { nextTurn } from "./play"
import type { Game } from "./types"

/**
 * Checks whether a raise is legal under Singapore Bridge rules.
 *
 * A raise is legal if:
 * - `betSize > game.BetSize` (strictly higher bid), OR
 * - `betSize === game.BetSize` AND the suit has higher priority
 *   (Spades > Hearts > Diamonds > Clubs)
 *
 * @param game       - Current game state
 * @param betSize    - Proposed bid (1–7)
 * @param bettedSuit - Proposed trump suit
 * @returns `true` if the raise beats the current bid
 */
export function isLegalRaise(game: Game, betSize: number, bettedSuit: string): boolean {
    const suitPriority: Record<string, number> = {
        Club: 0,
        Diamond: 1,
        Heart: 2,
        Spades: 3,
    }

    return (
        betSize > game.BetSize ||
        (betSize === game.BetSize && suitPriority[bettedSuit] > suitPriority[game.Trump])
    )
}

/**
 * Places a raise (bet) for the current player.
 *
 * Appends the raise to the betting history so the full log is preserved
 * for display. The bet winner is later resolved by scanning backwards
 * for the last non-pass entry.
 *
 * @param game       - Current game state (mutated in-place)
 * @param betSize    - Bid amount
 * @param bettedSuit - Trump suit for this hand
 */
export function raiseBet(game: Game, betSize: number, bettedSuit: string): void {
    const player = game.Players[game.WhoseTurn - 1]
    game.BetSize = betSize
    game.Trump = bettedSuit
    game.Moves.push({
        CardPlayed: { Rank: "", Value: betSize, Suit: bettedSuit, WonSet: false },
        PlayerID: player.ID,
    })
    nextTurn(game)
}

/**
 * Passes for the current player.
 *
 * Appends a pass to the betting log. Betting ends when 3 consecutive
 * passes follow the last raise (or everyone passes). The bet winner
 * is the last player who raised.
 *
 * @param game - Current game state (mutated in-place)
 */
export function passBet(game: Game): void {
    const player = game.Players[game.WhoseTurn - 1]
    game.Moves.push({
        CardPlayed: { Rank: "", Value: 0, Suit: "", WonSet: false },
        PlayerID: player.ID,
    })
    nextTurn(game)

    // Betting ends after 3 consecutive passes following the last raise
    // (or when everyone passes, e.g. at game start)
    const lastThree = game.Moves.slice(-3)
    if (game.Moves.length >= 4 && lastThree.every(m => m.CardPlayed.Value === 0)) {
        endBetting(game)
    }
}

/** Ends the betting phase and transitions to partner selection. */
function endBetting(game: Game): void {
    game.IsBettingPhase = false

    // Find the last non-pass raise (scan backwards through history)
    const lastRaise = [...game.Moves].reverse().find(m => m.CardPlayed.Value > 0)
    if (lastRaise) {
        game.BetWinner = game.Players[lastRaise.PlayerID - 1]
    } else {
        // Everyone passed — force minimum bet on P1
        game.BetWinner = game.Players[0]
        game.BetSize = 1
        game.Trump = "Club"
    }

    // Enter partner selection phase
    game.IsPartnerSelectionPhase = true
    game.Moves = []

    // Bots auto-select; human gets the UI picker
    if (game.BetWinner.IsBot) {
        autoSelectPartner(game)
    }
}

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
 * Resets the betting Moves to a single entry (the raise),
 * then advances to the next player.
 *
 * @param game       - Current game state (mutated in-place)
 * @param betSize    - Bid amount
 * @param bettedSuit - Trump suit for this hand
 */
export function raiseBet(game: Game, betSize: number, bettedSuit: string): void {
    const player = game.Players[game.WhoseTurn - 1]
    game.BetSize = betSize
    game.Trump = bettedSuit
    game.Moves = [
        {
            CardPlayed: { Rank: "", Value: betSize, Suit: bettedSuit, WonSet: false },
            PlayerID: player.ID,
        },
    ]
    nextTurn(game)
}

/**
 * Passes for the current player.
 *
 * Appends a pass (Value: 0) to the betting log. If all 4 players
 * have acted, the betting phase ends and partner selection begins.
 *
 * If the bet winner is a bot, partner is auto-selected immediately.
 * If human, the UI shows the partner selection screen.
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

    // All 4 players have acted — betting phase ends
    if (game.Moves.length === 4) {
        game.IsBettingPhase = false
        // The first mover is the bet winner (last raise before passes)
        game.BetWinner = game.Players[game.Moves[0].PlayerID - 1]

        // Enter partner selection phase
        game.IsPartnerSelectionPhase = true
        game.Moves = []

        // Bots auto-select; human gets the UI picker
        if (game.BetWinner.IsBot) {
            autoSelectPartner(game)
        }
    }
}

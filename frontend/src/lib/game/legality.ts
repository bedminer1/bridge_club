/** @file Card-legality rules for Singapore Bridge */

import type { Game, Card, Player } from "./types"

/**
 * Checks whether a card is **illegal** for the given player to play.
 *
 * Rules (Singapore Bridge):
 * 1. Not your turn → illegal
 * 2. Must follow the led suit if you have it
 * 3. Cannot **lead** trump unless trump has been played in a previous trick
 *    (you may still play trump in response to another suit when void)
 *
 * @returns `true` if the card cannot be played, `false` if it's legal
 */
export function isCardIllegal(game: Game, player: Player, card: Card): boolean {
    // Rule 1: Must be this player's turn
    if (game.WhoseTurn !== player.ID) {
        return true
    }

    // Rule 2: Follow suit
    if (game.TurnSuit !== "" && game.TurnSuit !== card.Suit) {
        const hasTurnSuit = player.Cards.some(c => c.Suit === game.TurnSuit)
        if (hasTurnSuit) {
            return true // Must follow suit when possible
        }
        // Void in led suit — can play anything (including trump)
        return false
    }

    // Rule 3: Cannot lead trump before it's been established
    if (!game.TrumpPlayed && card.Suit === game.Trump) {
        return true
    }

    return false
}

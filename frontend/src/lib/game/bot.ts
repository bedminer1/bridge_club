/** @file Bot AI: automatic betting, partner selection, and card play */

import { playCard, nextTurn, selectPartner } from "./play"
import { isCardIllegal } from "./legality"
import { doesCard1Beat } from "./cards"
import { raiseBet, passBet, isLegalRaise } from "./betting"
import type { Game, Card, Move, Player } from "./types"

// ── Betting ────────────────────────────────────────────────────────

/** Per-suit strength assessment used when choosing a bid suit. */
interface SuitStrength {
    numberOfCards: number
    /** Sum of (value - 10) for each J/Q/K/A in this suit */
    picturesValue: number
}

/**
 * Finds a player's strongest suit using the bot's bidding heuristic:
 * card count × 2 plus the value of its picture cards.
 *
 * Ties retain the first suit encountered in the player's hand, matching the
 * previous automatic-bidding behaviour.
 */
export function findStrongestSuit(player: Pick<Player, "Cards">): string {
    const strengths = new Map<string, SuitStrength>()
    for (const card of player.Cards) {
        const current = strengths.get(card.Suit)
        if (!current) {
            strengths.set(card.Suit, {
                numberOfCards: 1,
                picturesValue: card.Value > 10 ? card.Value - 10 : 0,
            })
        } else {
            current.numberOfCards++
            current.picturesValue += card.Value > 10 ? card.Value - 10 : 0
        }
    }

    let strongestSuit = "Club"
    let highestScore = 0
    for (const [suit, strength] of strengths) {
        const score = strength.numberOfCards * 2 + strength.picturesValue
        if (score <= highestScore) continue
        strongestSuit = suit
        highestScore = score
    }

    return strongestSuit
}

/** Returns the bidding-heuristic score for a particular suit. */
function suitStrengthScore(player: Pick<Player, "Cards">, suit: string): number {
    return player.Cards.reduce((score, card) => {
        if (card.Suit !== suit) return score
        return score + 2 + (card.Value > 10 ? card.Value - 10 : 0)
    }, 0)
}

/**
 * Bot betting strategy: scores each suit by (card count × 2 + picture value),
 * then bids the strongest suit at an appropriate level.
 *
 * - Score ≥ 16 → bet 3
 * - Score ≥ 13 → bet 2
 * - Otherwise  → bet 1
 *
 * If the calculated raise isn't legal (outranked), the bot passes.
 */
export function autoBet(game: Game): void {
    const player = game.Players[game.WhoseTurn - 1]
    const suit = findStrongestSuit(player)
    const score = suitStrengthScore(player, suit)
    const bestBet = {
        suit,
        betSize: score >= 16 ? 3 : score >= 13 ? 2 : 1,
    }

    if (isLegalRaise(game, bestBet.betSize, bestBet.suit)) {
        raiseBet(game, bestBet.betSize, bestBet.suit)
    } else {
        passBet(game)
    }
}

// ── Partner Selection ──────────────────────────────────────────────

/**
 * Bot partner selection: finds the player with the highest trump card
 * among the non-bet-winner players.
 *
 * **Fallback:** If no partner has any trump cards, picks the next
 * player cyclically (P2→P3, P3→P4, P4→P1, P1→P2).
 *
 * The partner card is set to the partner's highest trump (or first card
 * if they have no trump), then `selectPartner` is called to finalize teams.
 */
export function autoSelectPartner(game: Game): void {
    const betWinner = game.BetWinner

    // Search for the player with the strongest trump card
    let partner: Player | null = null
    let bestTrumpValue = 0
    for (const player of game.Players) {
        if (player === betWinner) continue
        for (const card of player.Cards) {
            if (card.Suit === game.Trump && card.Value > bestTrumpValue) {
                bestTrumpValue = card.Value
                partner = player
            }
        }
    }

    // Fallback: pick the next player in cyclic order
    if (!partner) {
        const nextId = betWinner.ID === 4 ? 1 : betWinner.ID + 1
        partner = game.Players[nextId - 1]
    }

    // Pick the partner's best trump card (or first card if none)
    const partnerCard = partner.Cards.reduce((best, card) =>
        card.Suit === game.Trump && card.Value > best.Value ? card : best,
        partner.Cards[0],
    )

    selectPartner(game, partnerCard)
}

// ── Card Play (Easy) ───────────────────────────────────────────────

/**
 * Easy bot: minimal strategy.
 *
 * - **Leading:** plays the strongest card available
 * - **Following:** tries to win with the strongest card; if can't win,
 *   plays the weakest card
 */
export function autoPlayCard(game: Game): void {
    const player = game.Players[game.WhoseTurn - 1]
    const moves = game.Moves
    const legal = player.Cards.filter(c => !isCardIllegal(game, player, c))

    // Leading: play strongest card
    if (moves.length === 0) {
        const strongest = legal.reduce((a, b) =>
            doesCard1Beat(game, a, b) ? a : b,
        )
        playCard(game, strongest, player)
        return
    }

    // Determine who's currently winning the trick
    const currentBest = findWinningMove(moves, game)
    const canWin = legal.filter(c => doesCard1Beat(game, c, currentBest.CardPlayed))
    const cardToPlay = canWin.length > 0
        ? strongestOf(legal.filter(c => doesCard1Beat(game, c, currentBest.CardPlayed)), game)
        : weakestOf(legal, game)

    playCard(game, cardToPlay, player)
}

// ── Card Play (Medium / Team-Aware) ────────────────────────────────

/**
 * Medium bot: team-aware strategy.
 *
 * - Tracks which cards have been played to estimate remaining strength
 * - If partner is winning: dumps weakest card
 * - If opponent is winning and it's the last play: wins with minimal card
 * - If leading: plays strongest un-played card, or weakest if none
 * - Otherwise: standard strongest/weakest logic
 */
export function autoPlayCardV2(game: Game): void {
    const player = game.Players[game.WhoseTurn - 1]
    const moves = game.Moves
    const legal = player.Cards.filter(c => !isCardIllegal(game, player, c))

    const weakest = legal.reduce((a, b) => (doesCard1Beat(game, a, b) ? b : a))

    // Compute strongest unplayed card per suit (for leading decisions)
    const played = game.Players.flatMap(p => p.PlayedCards)
    const unplayed = computeUnplayedStrengths(played)

    // Leading: play strongest unplayed card, else weakest
    if (moves.length === 0) {
        const hasStrongest = legal.find(c => c.Value === unplayed.get(c.Suit))
        if (hasStrongest) {
            playCard(game, hasStrongest, player)
        } else {
            playCard(game, weakest, player)
        }
        return
    }

    const currentBest = findWinningMove(moves, game)
    const teammateWinning = currentBest.PlayerID === player.Partner?.ID

    // Losing — need to win this trick
    if (!teammateWinning) {
        const canWin = legal.filter(c => doesCard1Beat(game, c, currentBest.CardPlayed))

        if (moves.length === 3 && canWin.length > 0) {
            // Last play: win with the weakest winning card
            const best = canWin.reduce((a, b) =>
                doesCard1Beat(game, a, b) ? b : a,
            )
            playCard(game, best, player)
            return
        }

        if (canWin.length === 0) {
            playCard(game, weakest, player)
            return
        }
    } else {
        // Partner winning — dump weakest card
        if (moves.length === 3) {
            playCard(game, weakest, player)
            return
        }
    }

    // Default: try to win strongly, otherwise dump weakest
    const canWin = legal.filter(c => doesCard1Beat(game, c, currentBest.CardPlayed))
    const cardToPlay = canWin.length > 0
        ? strongestOf(canWin, game)
        : weakestOf(legal, game)

    playCard(game, cardToPlay, player)
}

// ── Internal Helpers ───────────────────────────────────────────────

/** Finds the current winning Move in a trick. */
function findWinningMove(moves: Move[], game: Game): Move {
    return moves.reduce((best, move) =>
        doesCard1Beat(game, move.CardPlayed, best.CardPlayed) ? move : best,
    )
}

/** Returns the strongest card from a list by game rules. */
function strongestOf(cards: Card[], game: Game): Card {
    return cards.reduce((a, b) => (doesCard1Beat(game, a, b) ? a : b))
}

/** Returns the weakest card from a list by game rules. */
function weakestOf(cards: Card[], game: Game): Card {
    return cards.reduce((a, b) => (doesCard1Beat(game, a, b) ? b : a))
}

/**
 * Builds a map of suit → highest value still unplayed.
 * Used by the Medium bot for leading decisions.
 */
function computeUnplayedStrengths(played: Card[]): Map<string, number> {
    const playedBySuit = new Map<string, Set<number>>()
    for (const card of played) {
        if (!playedBySuit.has(card.Suit)) {
            playedBySuit.set(card.Suit, new Set())
        }
        playedBySuit.get(card.Suit)!.add(card.Value)
    }

    const unplayed = new Map<string, number>()
    for (const suit of ["Club", "Diamond", "Heart", "Spades"]) {
        const playedVals = playedBySuit.get(suit) ?? new Set()
        for (let val = 14; val >= 2; val--) {
            if (!playedVals.has(val)) {
                unplayed.set(suit, val)
                break
            }
        }
    }
    return unplayed
}

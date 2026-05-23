/** @file Core game actions: playing cards, resolving tricks, partner selection */

import type { Game, Card, Player, CompletedSet } from "./types"
import { findStrongestCard } from "./cards"

// ── Turn Management ────────────────────────────────────────────────

/** Advances `WhoseTurn` to the next player (1 → 2 → 3 → 4 → 1). */
export function nextTurn(game: Game): void {
    game.WhoseTurn = game.WhoseTurn === 4 ? 1 : game.WhoseTurn + 1
}

// ── Card Play & Trick Resolution ───────────────────────────────────

/**
 * Plays a card for a player during the play phase.
 *
 * Responsibilities:
 * - Guard: no-op if hand empty or wrong turn
 * - Track: lead suit, trump establishment, partner reveal
 * - Record: push move, remove from hand, add to played pile
 * - If 4 cards in the trick: resolve the trick
 *
 * @param game   - Current game state (mutated in-place)
 * @param card   - The card to play
 * @param player - The player playing it (must match game.WhoseTurn)
 */
export function playCard(game: Game, card: Card, player: Player): void {
    // Guard: nothing to play or wrong turn
    if (player.Cards.length === 0 || game.WhoseTurn !== player.ID) return

    // Track the led suit on the first move of a trick
    if (game.Moves.length === 0) {
        game.TurnSuit = card.Suit
    }

    // Mark trump as established when played (unlocks future trump leads)
    if (card.Suit === game.Trump) {
        game.TrumpPlayed = true
    }

    // Check if this card reveals the partner
    if (isPartnerCard(game, card)) {
        revealPartner(game, player)
    }

    // Record the move
    game.Moves.push({ PlayerID: player.ID, CardPlayed: card })

    // Remove card from hand, add to played pile
    removeFromHand(player, card)
    player.PlayedCards.push(card)

    // Advance turn
    nextTurn(game)

    // Resolve trick when all 4 players have played
    if (game.Moves.length === 4) {
        resolveTrick(game)
    }
}

// ── Partner Selection ──────────────────────────────────────────────

/**
 * Selects a partner card and sets up teams.
 *
 * Called when:
 * - A human clicks a card in the partner selection UI
 * - A bot calls `autoSelectPartner`
 *
 * This transitions the game from partner selection to play phase.
 *
 * @param game - Current game state (mutated in-place)
 * @param card - The card whose holder becomes the bet winner's partner
 */
export function selectPartner(game: Game, card: Card): void {
    game.PartnerCard = card

    // Find which player owns this card
    const owner = game.Players.find(p =>
        p.Cards.some(c => c.Suit === card.Suit && c.Value === card.Value)
    )!

    // Set up partnerships
    game.BetWinner.Partner = owner
    linkOpponents(game, game.BetWinner, owner)

    // Set up teams
    game.Team1 = [game.BetWinner, owner]
    game.Team2 = game.Players.filter(p => p !== game.BetWinner && p !== owner)

    // Transition to play phase
    game.IsPartnerSelectionPhase = false
    // Player to the left of the bet winner leads the first trick
    game.WhoseTurn = game.BetWinner.ID === 4 ? 1 : game.BetWinner.ID + 1
    game.Moves = []
    game.TurnSuit = ""
    game.CompletedSets = []
}

// ── Private Helpers ────────────────────────────────────────────────

/** True when the played card matches the pre-selected partner card. */
function isPartnerCard(game: Game, card: Card): boolean {
    return card.Suit === game.PartnerCard.Suit && card.Value === game.PartnerCard.Value
}

/**
 * Reveals the partner: links the bet winner to their partner
 * and links the two opponents together.
 */
function revealPartner(game: Game, player: Player): void {
    game.BetWinner.Partner = player
    linkOpponents(game, game.BetWinner, player)
}

/** Links the two non-partner players as each other's partners. */
function linkOpponents(game: Game, winner: Player, partner: Player): void {
    const opponents = game.Players.filter(p => p !== winner && p !== partner)
    opponents[0].Partner = opponents[1]
    opponents[1].Partner = opponents[0]
}

/** Removes a card from a player's hand by suit+value match. */
function removeFromHand(player: Player, card: Card): void {
    const idx = player.Cards.findIndex(c => c.Suit === card.Suit && c.Value === card.Value)
    if (idx !== -1) player.Cards.splice(idx, 1)
}

// ── Trick Resolution ───────────────────────────────────────────────

/**
 * Resolves a completed trick (4 cards played).
 *
 * - Finds the strongest card → determines the winner
 * - Awards a set to the winner's team
 * - Sets up the next trick (clear moves, set turn)
 * - Checks if either team has reached their win target
 */
function resolveTrick(game: Game): void {
    const cards = game.Moves.map(m => m.CardPlayed)
    const strongest = findStrongestCard(game, cards)
    strongest.WonSet = true

    const winnerId = game.Moves.find(m => m.CardPlayed === strongest)!.PlayerID
    const winner = game.Players[winnerId - 1]
    winner.Sets++

    // Record the completed set
    const completedSet: CompletedSet = {
        Cards: game.Moves.map(m => ({
            ...m.CardPlayed,
            WonSet: m.PlayerID === winnerId,
        })),
        WinnerID: winnerId,
        PlayerIDs: game.Moves.map(m => m.PlayerID),
    }
    game.CompletedSets.push(completedSet)

    // Prepare for next trick
    game.WhoseTurn = winner.ID
    game.PreviousMoves = game.Moves
    game.Moves = []
    game.TurnSuit = ""

    // Check win condition
    checkWinCondition(game, winnerId)
}

/**
 * Checks whether a team has reached their winning set target.
 *
 * Team 1 (bet winner's team) needs `6 + BetSize` sets.
 * Team 2 needs `8 - BetSize` sets.
 *
 * If the target is reached, the game ends and bot play stops.
 */
function checkWinCondition(game: Game, winnerId: number): void {
    const winnerInTeam1 = game.Team1.some(p => p.ID === winnerId)
    if (winnerInTeam1) {
        const total = game.Team1.reduce((sum, p) => sum + p.Sets, 0)
        const target = 6 + game.BetSize
        if (total >= target) endGame(game, "Team 1")
    } else {
        const total = game.Team2.reduce((sum, p) => sum + p.Sets, 0)
        const target = 8 - game.BetSize
        if (total >= target) endGame(game, "Team 2")
    }
}

/** Ends the game: disables bot play and records the winner. */
function endGame(game: Game, team: string): void {
    game.TurnOnBots = false
    game.Winner = team
}

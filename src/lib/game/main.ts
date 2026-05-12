import { autoPlayCard } from "./bot"

export function isCardIllegal(game: Game, player: Player, card: Card): boolean {
    if (game.WhoseTurn !== player.ID) {
        return true
    }

    if (game.TurnSuit !== "" && game.TurnSuit !== card.Suit) {
        const hasTurnSuit = player.Cards.some(c => c.Suit === game.TurnSuit)
        if (hasTurnSuit) {
            return true // Player must follow suit if possible
        } else {
            return false
        }
    }
        if (!game.TrumpPlayed && card.Suit === game.Trump) {
        return true
    }

    return false
}

export function doesCard1Beat(game: Game, c1: Card, c2: Card): boolean {
    if (c1.Suit === c2.Suit) return c1.Value > c2.Value
    if (c1.Suit === game.Trump && c2.Suit !== game.Trump) return true
    if (c1.Suit !== game.Trump && c2.Suit === game.Trump) return false
    if (c1.Suit === game.TurnSuit && c2.Suit !== game.TurnSuit) return true
    return false
}

export function findStrongestCard(game: Game, cards: Card[]) {
    let strongestCard: Card = cards[0]

    for (let card of cards) {
        if (doesCard1Beat(game, card, strongestCard)) {
            strongestCard = card
        } 
    }

    return strongestCard
}

export function nextTurn(game: Game) {
    game.WhoseTurn = game.WhoseTurn === 4 ? 1 : game.WhoseTurn + 1 
}

export function playCard(game: Game, card: Card, player: Player) {
    if (game.Moves.length === 0) {
        game.TurnSuit = card.Suit
    }
    if (card.Suit === game.Trump) {
        game.TrumpPlayed = true
    }
    if (card === game.PartnerCard) {
        game.BetWinner.Partner = player
        let opponents = game.Players.filter(p => p !== game.BetWinner && p !== player)
        opponents[0].Partner = opponents[1]
        opponents[1].Partner = opponents[0]
    }
    if (player.Cards.length === 0 || game.WhoseTurn !== player.ID) return

    game.Moves.push({
        PlayerID: player.ID,
        CardPlayed: card
    })

    const hand = player.Cards
    const index = hand.findIndex(c =>
        c.Suit === card.Suit && c.Value === card.Value
    )
    hand.splice(index, 1)
    player.PlayedCards.push(card)

    nextTurn(game)

    // check for end of stack
    if (game.Moves.length !== 4) { return }

    // scan for highest value card
    const cards = game.Moves.map(move => move.CardPlayed)
    const strongestCard = findStrongestCard(game, cards)
    strongestCard.WonSet = true

    let winnerID = game.Moves.find(move => move.CardPlayed === strongestCard)!.PlayerID
    let winner = game.Players[winnerID-1]
    winner.Sets++
    game.WhoseTurn = winner.ID
    game.Moves = []
    game.TurnSuit = ""

    // check for win
    let winnerInTeam1 = game.Team1.some(p => p.ID === winnerID)
    if (winnerInTeam1) {
        const team1Sets = game.Team1.reduce((sum, player) => sum + player.Sets, 0)
        const team1Target = 6 + game.BetSize
        if (team1Sets === team1Target) {
            game.TurnOnBots = false
            game.Winner = "Team 1"
        }
    } else {
        const team2Sets = game.Team2.reduce((sum, player) => sum + player.Sets, 0)
        const team2Target = 8 - game.BetSize
        if (team2Sets === team2Target) {
            game.TurnOnBots = false
            game.Winner = "Team 2"
        }
    }
}
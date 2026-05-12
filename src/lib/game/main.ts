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

export function playCard(game: Game, card: Card, player: Player) {
    if (game.Moves.length === 0) {
        game.TurnSuit = card.Suit
    }
    if (card.Suit === game.Trump) {
        game.TrumpPlayed = true
    }

    game.Moves.push({
        PlayerID: player.ID,
        CardPlayed: card
    })

    const hand = player.Cards
    const index = hand.findIndex(c =>
        c.Suit === card.Suit && c.Value === card.Value
    )
    hand.splice(index, 1)

    game.WhoseTurn = game.WhoseTurn === 4 ? 1 : game.WhoseTurn + 1 

    // check for end of stack
    if (game.Moves.length !== 4) { return }

    // scan for highest value card
    let stackSuit = game.Moves[0].CardPlayed.Suit
    game.Moves.sort((a,b) => {
        const aSuit = a.CardPlayed.Suit
        const bSuit = b.CardPlayed.Suit
        const aValue = a.CardPlayed.Value
        const bValue = b.CardPlayed.Value

        const aIsTrump = aSuit === game.Trump
        const bIsTrump = bSuit === game.Trump

        if (aIsTrump && !bIsTrump) return -1
        if (!aIsTrump && bIsTrump) return 1 // swap a and b

        const aIsStack = aSuit === stackSuit
        const bIsStack = bSuit === stackSuit
        if (aIsStack && !bIsStack) return -1
        if (!aIsStack && bIsStack) return 1

        if (aSuit === bSuit) return bValue - aValue
        return 0
    })

    let winnerID = game.Moves[0].PlayerID
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
            game.Winner = "Team 1"
        }
    } else {
        const team2Sets = game.Team2.reduce((sum, player) => sum + player.Sets, 0)
        const team2Target = 6 + game.BetSize
        if (team2Sets === team2Target) {
            game.Winner = "Team 2"
        }
    }
}
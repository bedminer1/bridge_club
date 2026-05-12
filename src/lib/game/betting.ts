export function raiseBet(game: Game, betSize: number, bettedSuit: string) {
    game.BetSize = betSize
    game.Trump = bettedSuit
    game.IsBettingPhase = false // for now, todo: implement passing and turns

    const betWinner = game.Players[game.WhoseTurn-1]
    const partner = autoFindPartner(game, betWinner) // option for manual
    const opponents = game.Players.filter(p => p !== betWinner && p !== partner)
    game.Team1 = [betWinner, partner]
    game.Team2 = opponents
}

// bots defaults to finding player holding highest value 
// trump suit card the they don't have
function autoFindPartner(game: Game, betWinner: Player) {
    let highestCard: Card | null = null
    let partner
    
    for (const player of game.Players) {
        if (player === betWinner) continue
        for (const card of player.Cards) {
            if (card.Suit === game.Trump) {
                if (!highestCard || card.Value > highestCard.Value) {
                    highestCard = card
                    partner = player
                }
            }
        }
    }

    return partner!
}
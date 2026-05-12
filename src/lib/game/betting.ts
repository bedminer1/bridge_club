import { autoFindPartner } from "./bot"
import { nextTurn } from "./main"

export function isLegalRaise(game: Game, betSize: number, bettedSuit: string) {
    const suitPriority = new Map<string, number>([
        ["Club", 0],
        ["Diamond", 1],
        ["Heart", 2],
        ["Spades", 3]
    ])
    
    return betSize > game.BetSize || betSize === game.BetSize && suitPriority.get(bettedSuit)! > suitPriority.get(game.Trump)!
}

export function raiseBet(game: Game, betSize: number, bettedSuit: string) {
    game.BetSize = betSize
    game.Trump = bettedSuit
    const player = game.Players[game.WhoseTurn-1]
    game.Moves = [{
        CardPlayed: {
            Rank: "",
            Value: betSize,
            Suit: bettedSuit
        },
        PlayerID: player.ID
    }]
    
    nextTurn(game)
}

export function passBet(game: Game) {
    const player = game.Players[game.WhoseTurn-1]
    game.Moves.push({
        CardPlayed: {
            Rank: "",
            Value: 0,
            Suit: "",
        },
        PlayerID: player.ID
    })
    nextTurn(game)

    if (game.Moves.length === 4) {
        game.IsBettingPhase = false
        const betWinner = game.Players[game.Moves[0].PlayerID-1]
        game.BetWinner = betWinner
        const partner = autoFindPartner(game, betWinner) // option for manual
        const opponents = game.Players.filter(p => p !== betWinner && p !== partner)
        
        game.Team1 = [betWinner, partner]
        game.Team2 = opponents
        game.Moves = []
        
        nextTurn(game)
        return
    }

}
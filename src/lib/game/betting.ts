import { autoFindPartner } from "./bot"
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
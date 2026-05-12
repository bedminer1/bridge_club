import { playCard } from "./main"
import { isCardIllegal, doesCard1Beat } from "./main"

// bots defaults to finding player holding highest value 
// trump suit card the they don't have
export function autoFindPartner(game: Game, betWinner: Player) {
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

export function autoPlayCard(game: Game) {
    setInterval(() => {
        
    }, 1000);
    const player = game.Players[game.WhoseTurn-1]
    const moves = game.Moves

    // First to play: play strongest card
    if (moves.length === 0) {
        let strongestCard: Card = player.Cards[0]

        for (let card of player.Cards) {
            if (!isCardIllegal(game, player, card) && doesCard1Beat(game, card, strongestCard)) {
                strongestCard = card
            } 
        }
        playCard(game, strongestCard, player)
        return
    } 

    const turnSuit = game.TurnSuit
    const playableCards = player.Cards.filter(card => !isCardIllegal(game, player, card))
    let currentWinningMove: Move = moves[0]
    for (let move of moves.slice(1)) {
        if (doesCard1Beat(game, move.CardPlayed, currentWinningMove.CardPlayed)) {
            currentWinningMove = move
        }
    }

    let cardToPlay: Card = playableCards[0]
    const winningCards = playableCards.filter(card => doesCard1Beat(game, card, currentWinningMove.CardPlayed))

    // TODO: logic for keeping track of cards played
    if (winningCards.length !== 0) {
        // play strongest
        for (let card of winningCards) {
            if (doesCard1Beat(game, card, cardToPlay)) {
                cardToPlay = card
            }
        }
    } else {
        // play weakest
        for (let card of playableCards) {
            if (doesCard1Beat(game, cardToPlay, card)) {
                cardToPlay = card
            }
        }
    }

    playCard(game, cardToPlay, player)
}
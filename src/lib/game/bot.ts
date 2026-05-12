import { playCard, nextTurn } from "./main"
import { isCardIllegal, doesCard1Beat } from "./main"
import { raiseBet, passBet, isLegalRaise } from "./betting"

export function autoBet(game: Game) {
    const player = game.Players[game.WhoseTurn-1]

    // analyze strength of each suit
    interface strength {
        numberOfCards: number
        picturesValue: number
    }

    let strengths = new Map<string, strength>()

    for (let card of player.Cards) {
        if (!strengths.has(card.Suit)) {
            strengths.set(card.Suit, {
                numberOfCards: 1,
                picturesValue: card.Value > 10 ? card.Value - 10 : 0
            })
            continue
        }

        const currStrength = strengths.get(card.Suit)!
        strengths.set(card.Suit, {
            numberOfCards: currStrength.numberOfCards + 1,
            picturesValue: currStrength.picturesValue + (card.Value > 10 ? card.Value - 10 : 0)
        })
    }

    // find strongest suit, determine max bet
    let betSize = 1
    let bettedSuit = "Club"
    let highestScore = 0
    for (let [suit, strength] of strengths) {
        const score = strength.numberOfCards * 2 + strength.picturesValue
        if (score <= highestScore) continue

        highestScore = score
        bettedSuit = suit
        
        if (score >= 16) {
            betSize = 3
        } else if (score >= 13) {
            betSize = 2
        }
    }

    if (isLegalRaise(game, betSize, bettedSuit)) {
        raiseBet(game, betSize, bettedSuit)
    } else {
        passBet(game) // pass
    }
}

// bots defaults to finding player holding highest value 
// trump suit card the they don't have
export function autoFindPartner(game: Game, betWinner: Player) {
    let highestCard: Card = game.Players.find(p => p !== betWinner)?.Cards[0]!
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

    game.PartnerCard = highestCard

    return partner!
}

export function autoPlayCard(game: Game) {
    const player = game.Players[game.WhoseTurn-1]
    const moves = game.Moves
    const playableCards = player.Cards.filter(card => !isCardIllegal(game, player, card))

    // First to play: play strongest card
    if (moves.length === 0) {
        let strongestCard: Card = playableCards[0]

        for (let card of playableCards) {
            if (doesCard1Beat(game, card, strongestCard)) {
                strongestCard = card
            } 
        }
        playCard(game, strongestCard, player)
        return
    } 

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

// adding teamwork
export function autoPlayCardV2(game: Game) {
    const player = game.Players[game.WhoseTurn-1]
    const moves = game.Moves
    const playableCards = player.Cards.filter(card => !isCardIllegal(game, player, card))
    
    const playedCards: Card[] = []
    for (let player of game.Players) {
        for (let card of player.PlayedCards) {
            playedCards.push(card)
        }
    }

    const playedValuesBySuit = new Map<string, Set<number>>()
    for (let card of playedCards) {
        if (!playedValuesBySuit.has(card.Suit)) {
            playedValuesBySuit.set(card.Suit, new Set())
        }
        playedValuesBySuit.get(card.Suit)!.add(card.Value)
    }

    const strongestUnplayedCards = new Map<string, number>()
    const suits = ["Club", "Diamond", "Heart", "Spades"]
    for (let suit of suits) {
        const played = playedValuesBySuit.get(suit) ?? new Set()
        for (let val = 14; val >= 2; val--) {
            if (!played.has(val)) {
                strongestUnplayedCards.set(suit, val)
                break
            }
        }
    }

    // if no cards played yet
    if (moves.length === 0) {
        let cardToPlay: Card = playableCards[0]

        // if has strongest playable card, play it
        for (let card of playableCards) {
            for (let [_, value] of strongestUnplayedCards) {
                if (card.Value === value) {
                    playCard(game, card, player)
                    return
                }
            }
        }

        // else play weakest card
       for (let card of playableCards) {
            if (doesCard1Beat(game, cardToPlay, card)) {
                cardToPlay = card
            }
        }
        playCard(game, cardToPlay, player)
        return
    } 


    // TEMPORARY IMPORT FROM VERSION 1
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
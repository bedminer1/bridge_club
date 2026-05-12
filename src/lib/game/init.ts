export function initGame(): Game {
    let cards: Card[] = []
    // generate cards
    for (let i = 0; i < 4; i++) { // suits
        for (let val = 2; val <= 14; val++) { // value
            cards.push({
                Rank: valueToRank.get(val)!,
                Value: val,
                Suit: suitEnum.get(i)!
            })
        }
    }

    // shuffle
    for (let i = cards.length - 1; i > 0; i--) {
        const j: number = Math.floor(Math.random() * (i + 1))
        ;[cards[i], cards[j]] = [cards[j], cards[i]]
    }

    // deal
    const hands: Card[][] = []
    for (let i = 0; i < 4; i++) {
        hands.push(cards.slice(i * 13, (i + 1) * 13))
    }

    // arrange cards
    for (const hand of hands) {
        hand.sort((a, b) => {
            const suitA = suitSortOrder.get(a.Suit)!
            const suitB = suitSortOrder.get(b.Suit)!
            if (suitA !== suitB) return suitA - suitB
            return a.Value - b.Value
        })
    }

    let players: Player[] = []
    for (let i = 1; i <= 4; i++) {
        players.push({
            ID: i,
            Cards: hands[i-1],
            PlayedCards: [],
            Partner: null,
            Sets: 0
        })
    }

    return {
        Players: players,
        Team1: [],
        BetWinner: players[0],
        PartnerCard: cards[0],
        Team2: [],
        Trump: "Club",
        BetSize: 0,
        IsBettingPhase: true,
        Moves: [],
        WhoseTurn: 1,
        CardsPlayed: [],
        TrumpPlayed: false,
        TurnSuit: "",
        Winner: ""
    }
}

const valueToRank = new Map([
    [2, "2"],
    [3, "3"],
    [4, "4"],
    [5, "5"],
    [6, "6"],
    [7, "7"],
    [8, "8"],
    [9, "9"],
    [10, "10"],
    [11, "J"],
    [12, "Q"],
    [13, "K"],
    [14, "A"],
])

const suitEnum = new Map([
    [0, "Club"],
    [1, "Diamond"],
    [2, "Heart"],
    [3, "Spades"],
])

const suitSortOrder = new Map<string, number>([
    ['Spades', 0],
    ['Heart', 1],
    ['Club', 2],
    ['Diamond', 3],
])
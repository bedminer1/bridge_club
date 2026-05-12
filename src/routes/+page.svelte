<script lang="ts">
    // cards
    interface Game {
        Players: Player[]
        Team1: number[] // winner of bet
        Team2: number[]
        Trump: string // trump suit
        Bet: number
    }

    interface Card {
        Rank: string // 2 to Ace
        Value: number // 2-14
        Suit: string
    }

    interface Player {
        ID: number // 1-4
        Cards: Card[]
        Sets: number
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

    function initGame(): Game {
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
                Sets: 0
            })
        }

        return {
            Players: players,
            Team1: [0, 1],
            Team2: [2, 3],
            Trump: "Spades",
            Bet: 1
        }
    }

    let game = initGame()

    function RaiseBet(betSize: number, suit: string) {

    }
</script>

<div class="flex gap-2">
    {#each game.Players as player}
    <div>
        <p>P{player.ID}:</p>
        {#each player.Cards as card}
        <div>{card.Rank} {card.Suit}</div>
        {/each}
    </div>
    {/each}
</div>


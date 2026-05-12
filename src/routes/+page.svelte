<script lang="ts">
    import { enhance } from "$app/forms";
    import * as Select from "$lib/components/ui/select/index.js";
    import { Input } from "$lib/components/ui/input/index.js";
    import { Button } from "$lib/components/ui/button/index.js";

    // cards
    interface Game {
        Players: Player[]
        Team1: number[] // winner of bet
        Team2: number[]
        Trump: string // trump suit
        BetSize: number
        IsBettingPhase: boolean
        Moves: Move[]
        WhoseTurn: number
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

    interface Move {
        CardPlayed: Card
        PlayerID: number
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
            Trump: "Club",
            BetSize: 1,
            IsBettingPhase: true,
            Moves: [],
            WhoseTurn: 1
        }
    }

    let game = initGame()

    let betSize: number
    let bettedSuit: string = "Club"

    function raiseBet() {
        game.BetSize = betSize
        game.Trump = bettedSuit
        game.IsBettingPhase = false // for now
    }

    function playCard(card: Card, playerID: number) {
        if (game.IsBettingPhase) {
            console.log("Can't bet during betting phase")
            return
        }

        if (game.WhoseTurn !== playerID) {
            console.log("Can't play when not your turn")
            return
        }

        game.Moves.push({
            PlayerID: playerID,
            CardPlayed: card
        })
        game.WhoseTurn = game.WhoseTurn === 4 ? 1 : game.WhoseTurn + 1 

        // check for end of stack
        if (game.Moves.length === 4) {
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
        }
    }
</script>

<div class="flex flex-col gap-10 w-full h-screen justify-center items-center">
    <div>
        <p>Trump Suit: {game.Trump}</p>
        <p>Bet Size: {game.BetSize}</p>
    </div>
    <div class="flex gap-10">
        {#each game.Players as player}
        <div>
            <p>P{player.ID}:</p>
            {#each player.Cards as card}
            <div>{card.Rank} {card.Suit}</div>
            {/each}
        </div>
        {/each}
    </div>

    {#if game.IsBettingPhase}
    <div class="flex flex-col justify-center items-center gap-2">
        <div class="flex gap-2">
            <Input bind:value={betSize} type="number" placeholder="1 to 7"/>
    
            <Select.Root type="single" bind:value={bettedSuit}>
            <Select.Trigger class="w-[180px]">
                {bettedSuit}
            </Select.Trigger>
            <Select.Content>
                <Select.Item value="Club">Club</Select.Item>
                <Select.Item value="Diamond">Diamond</Select.Item>
                <Select.Item value="Heart">Heart</Select.Item>
                <Select.Item value="Spades">System</Select.Item>
            </Select.Content>
            </Select.Root>
        </div>
        <div class="flex gap-2">
            <Button>Pass</Button>
            <Button onclick={raiseBet}>Raise</Button>
        </div>
    </div>
    {/if}
</div>


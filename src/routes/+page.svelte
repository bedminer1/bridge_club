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
        Player: number
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
            BetSize: 1,
            IsBettingPhase: true,
        }
    }

    let game = initGame()

    let betSize: number
    let bettedSuit: string = "Club"

    function raiseBet() {
        game.BetSize = betSize
        game.Trump = bettedSuit
        game.IsBettingPhase = false
    }

    function playCard(card: Card) {

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


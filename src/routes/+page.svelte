<script lang="ts">
    import * as Select from "$lib/components/ui/select/index.js";
    import { Input } from "$lib/components/ui/input/index.js";
    import { Button } from "$lib/components/ui/button/index.js";

    import { initGame } from "$lib/game/init";
    import { raiseBet } from "$lib/game/betting";

    let game = $state(initGame())

    let betSize: number = $state(1)
    let bettedSuit: string = $state("Club")

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

        const player = game.Players[playerID - 1]
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

        // check for win
    }
</script>

<div class="flex flex-col gap-10 w-full h-screen justify-center items-center">
    <div>
        <p>Trump Suit: {game.Trump}</p>
        <p>Bet Size: {game.BetSize}</p>
        <p>Who's Turn: Player {game.WhoseTurn}</p>
    </div>

    <div>
        {#if !game.IsBettingPhase}
        <p>Team 1: {game.Team1[0].ID}, {game.Team1[1].ID}</p>
        <p>Team 2: {game.Team2[0].ID}, {game.Team2[1].ID}</p>
        {/if}
    </div>
    <div>
        {#each game.Moves as move}
        <p>Player {move.PlayerID} played {move.CardPlayed.Rank} {move.CardPlayed.Suit}</p>
        {/each}
    </div>
    <div class="flex gap-10">
        {#each game.Players as player}
        <div>
            <p>P{player.ID}:</p>
            <p>Sets: {player.Sets}</p>
            <div class="flex flex-col gap-2">
                {#each player.Cards as card}
                <Button onclick={()=>playCard(card, player.ID)}>
                    {card.Rank} {card.Suit}
                </Button>
                {/each}
            </div>
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
            <Button onclick={()=>raiseBet(game, betSize, bettedSuit)}>Raise</Button>
        </div>
    </div>
    {/if}
</div>


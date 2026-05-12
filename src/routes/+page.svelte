<script lang="ts">
    import * as Select from "$lib/components/ui/select/index.js";
    import { Input } from "$lib/components/ui/input/index.js";
    import { Button } from "$lib/components/ui/button/index.js";

    import { initGame } from "$lib/game/init";
    import { raiseBet } from "$lib/game/betting";
    import { isCardIllegal, playCard } from "$lib/game/main";

    let game = $state(initGame())

    // form inputs
    let betSize: number = $state(1)
    let bettedSuit: string = $state("Club")
    let desiredCard: Card | undefined = $state()
</script>

<div class="flex flex-col gap-10 w-full h-screen justify-center items-center">
    <div class="grid grid-cols-2 gap-2">
        <p>Trump Suit: {game.Trump}</p>
        <p>Bet Size: {game.BetSize}</p>
        <p>Who's Turn: Player {game.WhoseTurn}</p>
        <p>Current Suit: {game.TurnSuit}</p>
        <p>Winners: {game.Winner}</p>
    </div>

    {#if !game.IsBettingPhase}
    <div>
        <p>Team 1: {game.Team1[0].ID}, {game.Team1[1].ID}</p>
        <p>Team 2: {game.Team2[0].ID}, {game.Team2[1].ID}</p>
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
                <Button 
                    disabled={isCardIllegal(game, player, card)}
                    onclick={()=>playCard(game, card, player)}>
                    {card.Rank} {card.Suit}
                </Button>
                {/each}
            </div>
        </div>
        {/each}
    </div>
    {:else}
     <div>
            <p>P{game.Players[0].ID}:</p>
            <p>Sets: {game.Players[0].Sets}</p>
            <div class="flex flex-col gap-2">
                {#each game.Players[0].Cards as card}
                <Button>
                    {card.Rank} {card.Suit}
                </Button>
                {/each}
            </div>
        </div>

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
                    <Select.Item value="Spades">Spades</Select.Item>
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


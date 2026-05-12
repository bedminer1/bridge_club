<script lang="ts">
    import * as Select from "$lib/components/ui/select/index.js";
    import { Input } from "$lib/components/ui/input/index.js";
    import { Button } from "$lib/components/ui/button/index.js";
    import { Switch } from "$lib/components/ui/switch/index.js";
    import { Label } from "$lib/components/ui/label/index.js";
    import * as Dialog from "$lib/components/ui/dialog/index.js";

    import { initGame } from "$lib/game/init";
    import { raiseBet, passBet } from "$lib/game/betting";
    import { isCardIllegal, playCard } from "$lib/game/main";
    import { autoBet, autoPlayCard } from "$lib/game/bot";

    let game = $state(initGame())

    // form inputs
    let betSize: number = $state(1)
    let bettedSuit: string = $state("Club")
    let desiredCard: Card | undefined = $state()
    let hiddenMode = $state(true)

    $effect(() => {
        if (game)
        setInterval(() => {
            if (game.WhoseTurn !== 1) {
                if (game.IsBettingPhase) {
                    autoBet(game)
                } else {
                    autoPlayCard(game)
                }
            }
        }, 3000);
    })
</script>

<div class="flex flex-col gap-10 w-full h-screen justify-center items-center">
    <Dialog.Root open={game.Winner !== ""}>
    <Dialog.Content>
        <Dialog.Header>
        <Dialog.Title>{game.Winner} Won!</Dialog.Title>
        <Dialog.Description>
            {game.Winner} has won {game.Winner ===  "Team 1" ? 6 + game.BetSize : 8 - game.BetSize} sets to win the game!
        </Dialog.Description>
        </Dialog.Header>
    </Dialog.Content>
    </Dialog.Root>
    <div class="flex justify-end w-full">
        <div class="flex items-center space-x-2">
            <Label for="hidden-mode">Hidden Mode</Label>
            <Switch id="hidden-mode" bind:checked={hiddenMode}/>
        </div>
    </div>
    <div class="grid grid-cols-2 gap-2">
        <p>Trump Suit: {game.Trump}</p>
        <p>Bet Size: {game.BetSize}</p>
        <p>Who's Turn: Player {game.WhoseTurn}</p>
        <p>Current Suit: {game.TurnSuit}</p>
        {#if !game.IsBettingPhase}
        <p>Bet Winner: Player {game.BetWinner.ID}</p>
        <p>Partner Card: {game.PartnerCard.Rank} {game.PartnerCard.Suit}</p>
        {/if}
    </div>

    {#if game.IsBettingPhase}
    <div>
        {#each game.Moves as move}
        {#if move.CardPlayed.Value === 0}
        <p>Player {move.PlayerID} passed</p>
        {:else}
        <p>Player {move.PlayerID} raised {move.CardPlayed.Value} {move.CardPlayed.Suit}</p>
        {/if}
        {/each}
    </div>
    {:else}
    <div>
        {#each game.Moves as move}
        <p>Player {move.PlayerID} played {move.CardPlayed.Rank} {move.CardPlayed.Suit}</p>
        {/each}
    </div>
    {/if}

    {#if !game.IsBettingPhase}
    <div>
        <p>Team 1: needs {6 + game.BetSize}</p>
        <p>Team 2: needs {8 - game.BetSize}</p>
        <Button onclick={()=>autoPlayCard(game)}>Autoplay move</Button>
    </div>
    <div class="flex gap-10">
        {#each game.Players as player}
        <div>
            <p>P{player.ID}:</p>
            <p>Sets: {player.Sets}</p>
            <div class="flex flex-col gap-2">
                {#each !hiddenMode || player.ID === 1 ? player.Cards : []  as card}
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
        <div class="flex gap-10">
            {#each game.Players as player}
            <div>
                <p>P{player.ID}:</p>
                <p>Sets: {player.Sets}</p>
                <div class="flex flex-col gap-2">
                    {#each !hiddenMode || player.ID === 1 ? player.Cards : []  as card}
                    <Button>
                        {card.Rank} {card.Suit}
                    </Button>
                    {/each}
                </div>
            </div>
            {/each}
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
                <Button onclick={()=>passBet(game)}>Pass</Button>
                <Button onclick={()=>raiseBet(game, betSize, bettedSuit)}>Raise</Button>
            </div>
        </div>
    {/if}


</div>


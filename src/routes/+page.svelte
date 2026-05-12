<script lang="ts">
    import * as Select from "$lib/components/ui/select/index.js";
    import { Input } from "$lib/components/ui/input/index.js";
    import { Button } from "$lib/components/ui/button/index.js";
    import { Switch } from "$lib/components/ui/switch/index.js";
    import { Label } from "$lib/components/ui/label/index.js";
    import * as Dialog from "$lib/components/ui/dialog/index.js";
    import { Separator } from "$lib/components/ui/separator/index.js";

    import PokerCard from "./PokerCard.svelte";

    import { initGame } from "$lib/game/init";
    import { raiseBet, passBet, isLegalRaise } from "$lib/game/betting";
    import { isCardIllegal, playCard } from "$lib/game/main";
    import { autoBet, autoPlayCard, autoPlayCardV2 } from "$lib/game/bot";

    let game = $state(initGame())

    // form inputs
    let betSize: number = $state(1)
    let bettedSuit: string = $state("Club")
    let hiddenMode = $state(true)
    let difficulty = $state("Easy")

    $effect(() => {
        if (game)
        setInterval(() => {
            if (game.WhoseTurn !== 1) {
                if (game.IsBettingPhase) {
                    autoBet(game)
                } else {
                    if (difficulty === "Easy") {
                        autoPlayCard(game)
                    } else if (difficulty === "Medium") {
                        autoPlayCardV2(game)
                    }
                }
            }
        }, 3000);
    })
</script>


<div class="flex flex-col gap-10 w-full h-screen  items-center overflow-auto">
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
    <div class="flex justify-end w-full p-3 gap-4">
        <div class="flex flex-col gap-2">
            <div class="flex items-center gap-2">
                <Label for="difficulty">Difficulty:</Label>
                <Select.Root type="single" bind:value={difficulty} disabled={!game.IsBettingPhase}>
                    <Select.Trigger class="w-[100px]">
                        {difficulty}
                    </Select.Trigger>
                    <Select.Content>
                        <Select.Item value="Easy">Easy</Select.Item>
                        <Select.Item value="Medium">Medium</Select.Item>
                        <Select.Item value="Hard" disabled={true}>Hard</Select.Item>
                    </Select.Content>
                </Select.Root>
            </div>
            <div class="flex items-center space-x-2">
                <Label for="hidden-mode">Hidden Mode: </Label>
                <Switch id="hidden-mode" bind:checked={hiddenMode}/>
            </div>
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
        <p>Team 1 needs {6 + game.BetSize}</p>
        <p>Team 2 needs {8 - game.BetSize}</p>
    </div>
    <div class="flex flex-col gap-10">
        {#each game.Players as player}
        <div>
            <p>Player {player.ID} ({player.Sets} sets) </p>
            <div class="flex h-[100px]">
                {#each !hiddenMode || player.ID === 1 ? player.Cards : []  as card, index}
                <button
                    disabled={isCardIllegal(game, player, card)}
                    onclick={()=>playCard(game, card, player)}>
                    <div class="relative ml-[-1rem] z-{index} hover:z-40 hover:-translate-y-1">
                        <PokerCard card={card} isIllegal={isCardIllegal(game, player, card)}/>
                    </div>
                </button>
                {/each}
                {#if !hiddenMode}
                <Separator orientation="vertical" class="mx-10 h-full"/>
                    {#each player.PlayedCards as card, index}
                     <button 
                        disabled={true}>
                        <div class="relative ml-[-1rem] z-{index} hover:z-40 hover:-translate-y-1 disabled:opacity-50">
                            <PokerCard card={card} isIllegal={true}/>
                        </div>
                    </button>
                    {/each}
                {/if}
            </div>
        </div>
        {/each}
    </div>
    {:else}
        <div class="flex flex-col gap-10">
            {#each hiddenMode ? [game.Players[0]] : game.Players as player}
            <div class="flex flex-col h-[100px]">
                <p class="mb-2">Player {player.ID}</p>
                <div class="flex pl-4">
                    {#each !hiddenMode || player.ID === 1 ? player.Cards : []  as card, index}
                        <div class="relative ml-[-1rem] z-{index} transition-transform hover:z-40 hover:-translate-y-1">
                            <PokerCard card={card} isIllegal={false}/>
                        </div>
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
                <Button 
                onclick={()=>raiseBet(game, betSize, bettedSuit)}
                disabled={!isLegalRaise(game, betSize, bettedSuit)}>Raise</Button>
            </div>
        </div>
    {/if}


</div>


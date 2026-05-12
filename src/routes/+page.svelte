<script lang="ts">
    import * as Select from "$lib/components/ui/select/index.js";
    import { Input } from "$lib/components/ui/input/index.js";
    import { Button } from "$lib/components/ui/button/index.js";
    import { Separator } from "$lib/components/ui/separator/index.js";
    import * as Dialog from "$lib/components/ui/dialog/index.js";
    import * as Form from "$lib/components/ui/form/index.js";

    import { enhance } from "$app/forms";
    import PokerCard from "$lib/components/poker-card.svelte";
    import HandDisplay from "$lib/components/hand-display.svelte";
    
    import { suitToSymbol } from "$lib/utils"
    import { initGame } from "$lib/game/deck";
    import { raiseBet, passBet, isLegalRaise } from "$lib/game/betting";
    import { playCard, selectPartner } from "$lib/game/play";
    import { isCardIllegal } from "$lib/game/legality";
    import { autoBet, autoPlayCard, autoPlayCardV2 } from "$lib/game/bot";
    import { headerState } from "$lib/game/header-state.svelte";

    let { data } = $props()
    let { username, userID } = $state(data)

    let game = $state(initGame(username))

    // user info
    let loggedIn: boolean = $derived(userID === 0 ? false : true)
    let openSaveDialog: boolean = $state(false)

    function onlogout() {
        loggedIn = false
    }

    // Sync reactive game state to shared header state
    $effect(() => { headerState.game = game })

    let isLightMode = $state(false)
    $effect(() => { headerState.isLightMode = isLightMode })

    // Sync user info to header
    $effect(() => { headerState.username = username })
    $effect(() => { headerState.loggedIn = loggedIn })

    let userTeam = $derived(game.Team1.some(p => p.ID === 1) ? game.Team1 : game.Team2)
	let wonMatch = $derived(game.Winner === "Team 1" && userTeam === game.Team1 ||
	               game.Winner === "Team 2" && userTeam === game.Team2 ? 1 : 0)
    let partner = $derived(userTeam.find(p => p.ID !== 1)?.ID ?? 0)

    // form inputs
    let betSize: number = $state(1)
    let bettedSuit: string = $state("Club")

    const suitOrder: Record<string, number> = { Spades: 0, Heart: 1, Club: 2, Diamond: 3 }
    let remainingDeck = $derived(
        game.FullDeck
            .filter(fc => !game.Players[0].Cards.some(pc => pc.Suit === fc.Suit && pc.Value === fc.Value))
            .sort((a, b) => {
                const suitDiff = (suitOrder[a.Suit] ?? 0) - (suitOrder[b.Suit] ?? 0)
                if (suitDiff !== 0) return suitDiff
                return a.Value - b.Value
            })
    )

    const playerIDToColor = new Map<number, string>([
        [1, "[var(--red)]"],
        [2, "[var(--blue)]"],
        [3, "[var(--yellow)]"],
        [4, "[var(--green)]"],
    ])

    $effect(() => {
        if (game.Winner !== "") {
            openSaveDialog = true
            return
        }
        if (!game || headerState.botSpeed == undefined || game.WhoseTurn === 1 || !game.TurnOnBots) return

        const interval = setInterval(() => {
            if (game.IsBettingPhase) {
                autoBet(game)
            } else {
                if (headerState.difficulty === "Easy") {
                    autoPlayCard(game)
                } else if (headerState.difficulty === "Medium") {
                    autoPlayCardV2(game)
                }
            }
        }, headerState.botSpeed * 1000);

        return () => clearInterval(interval)
    })
</script>


<div class="flex flex-col gap-6 w-full min-h-screen items-center px-4 pt-20 pb-8">

    <div class="text-2xl text-muted-foreground">
        <p>Player {game.WhoseTurn}'s turn</p>
    </div>

{#if game.IsPartnerSelectionPhase}
    <div class="flex flex-col gap-4 items-center">
        <p class="text-xl">Select a partner card</p>
        <p class="text-sm opacity-70">Choose any card you don't own — the player holding it becomes your partner</p>
        <div class="flex flex-wrap gap-1 justify-center max-w-3xl">
            {#each remainingDeck as card}
                <button onclick={() => selectPartner(game, card)}
                    class="transition-transform brightness-105 dark:brightness-95 hover:brightness-130 dark:hover:brightness-120 hover:shadow-accent hover:shadow-xl/30 hover:-translate-y-1 active:brightness-125 active:shadow-accent rounded-sm">
                    <PokerCard card={card} isIllegal={false} minify={true} />
                </button>
            {/each}
        </div>
    </div>
{:else}
    <!-- Play area table -->
    <div class="flex flex-col gap-4 w-full max-w-3xl">
        <!-- Game info strip -->
        <div class="flex flex-wrap gap-x-4 gap-y-1 text-xs text-muted-foreground px-1">
            {#if game.BetSize > 0 || !game.IsBettingPhase}
            <span>Trump <strong class="text-accent font-medium">{suitToSymbol.get(game.Trump)} {game.Trump}</strong></span>
            <span>Bet <strong class="text-foreground font-medium">{game.BetSize}</strong></span>
            {/if}
            {#if !game.IsBettingPhase}
            <span>Winner <strong class="text-foreground font-medium">P{game.BetWinner.ID}</strong></span>
            <span>Partner <strong class="text-accent font-medium">{game.PartnerCard.Rank}{suitToSymbol.get(game.PartnerCard.Suit)}</strong></span>
            <span class="text-muted-foreground">|</span>
            <span>Set <strong class="text-foreground font-medium">{game.Players.reduce((s,p) => s + p.Sets, 0)}/13</strong></span>
            {/if}
        </div>

        <div class="rounded-xl border border-border bg-card/50 p-4 sm:p-6">
    {#if game.IsBettingPhase}
    <div class="flex flex-col items-center gap-1">
        {#each game.Moves.slice(-3) as move}
            <div class="flex items-center gap-2 {move === game.Moves[game.Moves.length - 1] ? 'text-base font-medium' : 'text-xs text-muted-foreground/60'}">
                <span class="text-{playerIDToColor.get(move.PlayerID)}">P{move.PlayerID}</span>
                {#if move.CardPlayed.Value === 0}
                    <span>passed</span>
                {:else}
                    <span>raised <strong>{move.CardPlayed.Value} {move.CardPlayed.Suit}</strong></span>
                {/if}
            </div>
        {/each}
        {#if game.Moves.length < 4}
            <div class="text-xs text-muted-foreground/40 mt-1">
                waiting for P{game.WhoseTurn}...
            </div>
        {/if}
        {#if game.Moves.length === 0}
            <div class="text-xs text-muted-foreground/40">
                P{game.WhoseTurn} to bet
            </div>
        {/if}
    </div>
    {:else}
    <div class="flex justify-center relative h-21 w-full">
        <div class="flex gap-2 mx-auto">
            {#each game.Moves as move}
            <div class="flex flex-col items-center">
                <PokerCard card={move.CardPlayed} isIllegal={false} minify={false} />
                <p class="text-{playerIDToColor.get(move.PlayerID)}">P{move.PlayerID}</p>
            </div>
            {/each}
        </div>
    
        {#if game.PreviousMoves.length !== 0} 
        <div class="absolute right-2 bottom-2 sm:right-1/6 sm:bottom-1/4 flex pl-4">
            {#each game.PreviousMoves as move, index}
                <HandDisplay index={index}>
                    <PokerCard card={move.CardPlayed} isIllegal={false} minify={true} />
                    <p class="text-xs text-{playerIDToColor.get(move.PlayerID)}">P{move.PlayerID}</p>
                </HandDisplay>
            {/each}
        </div>
        {/if}
    </div>
    {/if}


    {#if !game.IsBettingPhase}
    <!-- MAIN PHASE -->
    <div class="flex flex-col gap-10">
        {#each headerState.hiddenMode ? [game.Players[0]] : game.Players as player}
        <div>
            <div class="flex gap-2">
                <p class="text-{playerIDToColor.get(player.ID)}">{player.Username} ({player.Sets} sets) </p>
                {#if !headerState.hiddenMode && player.Partner !== null}
                <p>| Partner is Player {player.Partner?.ID}</p>
                {/if}
            </div>
            
            <div class="flex h-[100px] pl-4">
                {#each player.Cards  as card, index}
                <button
                    disabled={isCardIllegal(game, player, card)}
                    onclick={()=>playCard(game, card, player)}
                    class="text-left">
                    <HandDisplay index={index}>
                        <PokerCard card={card} isIllegal={isCardIllegal(game, player, card)} minify={false}/>
                    </HandDisplay>
                </button>
                {/each}
                {#if !headerState.hiddenMode}
                <Separator orientation="vertical" class="mx-10 h-full"/>
                    {#each player.PlayedCards as card, index}
                     <button 
                        disabled={true}>
                        <HandDisplay index={index}>
                            <PokerCard card={card} isIllegal={true} minify={false}/>
                        </HandDisplay>
                    </button>
                    {/each}
                {/if}
            </div>
        </div>
        {/each}

        {#if headerState.hiddenMode}
        <div class="flex gap-4">
            {#each game.Players.slice(1, 4) as player, index}
                <div class="flex gap-2">
                <p class="text-{playerIDToColor.get(player.ID)}">Player {player.ID} ({player.Sets} sets) </p>
                </div>

                {#if index < 2}
                <Separator orientation="vertical" />
                {/if}
            {/each}
        </div>
        {/if}
    </div>


    {:else} 
    <!-- BETTING PHASE -->
        <div class="flex flex-col gap-10">
            {#each headerState.hiddenMode ? [game.Players[0]] : game.Players as player}
            <div class="flex flex-col h-[100px]">
                <p class="mb-2 text-{playerIDToColor.get(player.ID)}">{player.Username}</p>
                <div class="flex pl-4">
                    {#each !headerState.hiddenMode || player.ID === 1 ? player.Cards : []  as card, index}
                        <HandDisplay index={index}>
                            <PokerCard card={card} isIllegal={false} minify={false}/>
                        </HandDisplay>
                    {/each}
                </div>
            </div>
            {/each}
        </div>

        <div class="flex flex-col justify-center gap-2">
            <div class="flex flex-col gap-2 items-start w-[45%]">
                <div class="flex gap-2 w-full">
                    <Input bind:value={betSize} class="text-center numberInput flex-1" type="number" placeholder="1-7"/>
                    <Select.Root type="single" bind:value={bettedSuit}>
                    <Select.Trigger class="flex-[3]">
                        <p class="text-sm">{suitToSymbol.get(bettedSuit)} {bettedSuit}</p>
                    </Select.Trigger>
                    <Select.Content>
                        <Select.Item value="Club">♣ Club</Select.Item>
                        <Select.Item value="Diamond">♦ Diamond</Select.Item>
                        <Select.Item value="Heart">♥ Heart</Select.Item>
                        <Select.Item value="Spades">♠ Spades</Select.Item>
                    </Select.Content>
                    </Select.Root>
                </div>
                <div class="flex gap-2 w-full">
                    <Button class="flex-1" onclick={()=>passBet(game)}>Pass</Button>
                    <Button 
                    variant="destructive"
                    onclick={()=>raiseBet(game, betSize, bettedSuit)}
                    disabled={!isLegalRaise(game, betSize, bettedSuit)}
                    class="flex-1"
                    >Raise</Button>
                </div>
            </div>
        </div>
    {/if}
        </div>
    </div>
{/if}
</div>

<Dialog.Root onOpenChange={()=>openSaveDialog = true} open={openSaveDialog}>
    <!-- <Dialog.Trigger>
        <Button>
            For Testing, Ignore
        </Button>
    </Dialog.Trigger> -->
    <Dialog.Content class="w-[40%]">
        <Dialog.Header>
        <Dialog.Title>{game.Winner} Won!</Dialog.Title>
        <Dialog.Description>
            <p class="mb-4">
                {game.Winner} has won {game.Winner ===  "Team 1" ? 6 + game.BetSize : 8 - game.BetSize} sets to win the game!
            </p>

            {#if loggedIn}
            <form action="?/saveMatch" 
            method="POST" 
            class="flex flex-col items-end" 
            use:enhance={() => {openSaveDialog = false}}>

                <!-- Metadata -->
	            <input type="hidden" name="date" value={Date.now()}>
	            <input type="hidden" name="botDifficulty" bind:value={headerState.difficulty}>
                
                <!-- User Info -->
                <input type="hidden" name="userID" bind:value={userID}>

                <!-- Betting Info -->
                <input type="hidden" name="trumpSuit" value={game.Trump}>
                <input type="hidden" name="betSize" value={game.BetSize}>
                <input type="hidden" name="betWinner" value={game.BetWinner.ID}>

                <!-- Match Result -->
                <input type="hidden" name="partner" value={partner}>
                <input type="hidden" name="wonMatch" value={wonMatch}>

                <!-- Sets Won -->
                {#each game.Players as player, i}
                    <input type="hidden" name={"player" + (i + 1) + "Sets"} value={player.Sets}>
                {/each}

                <!-- Hands (as JSON or comma-separated values) -->
                 {#each game.Players as player, i}
                    <input type="hidden" name={"player" + (i + 1) + "Hand"} value={JSON.stringify(player.PlayedCards)}>
                {/each}

                <Form.Button class="w-[60px] mt-4">
                    Save
                </Form.Button>
            </form>
        {/if}
        </Dialog.Description>
        </Dialog.Header>
    </Dialog.Content>
</Dialog.Root>
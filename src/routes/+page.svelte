<script lang="ts">
    import * as Select from "$lib/components/ui/select/index.js";
    import { Input } from "$lib/components/ui/input/index.js";
    import { Button } from "$lib/components/ui/button/index.js";
    import { Switch } from "$lib/components/ui/switch/index.js";
    import { Label } from "$lib/components/ui/label/index.js";
    import * as Dialog from "$lib/components/ui/dialog/index.js";
    import { Separator } from "$lib/components/ui/separator/index.js";
    import * as Popover from "$lib/components/ui/popover/index.js";
    import * as Form from "$lib/components/ui/form/index.js";

    import { enhance } from "$app/forms";
    import { Info, Settings, LogIn } from "@lucide/svelte"
    import PokerCard from "./PokerCard.svelte";
    import HandDisplay from "./HandDisplay.svelte";
    
    import { suitToSymbol } from "$lib/utils"
    import { initGame } from "$lib/game/init";
    import { raiseBet, passBet, isLegalRaise } from "$lib/game/betting";
    import { isCardIllegal, playCard } from "$lib/game/main";
    import { autoBet, autoPlayCard, autoPlayCardV2 } from "$lib/game/bot";

    let game = $state(initGame())

    let loggedIn: boolean = $state(false)
    let closeDialogue: boolean = $state(false)
    let username: string = $state("")
    let password: string = $state("")

    // form inputs
    let betSize: number = $state(1)
    let bettedSuit: string = $state("Club")
    let hiddenMode = $state(true)
    let difficulty = $state("Medium")
    let botSpeed = $state(3)

    $effect(() => {
        if (!game || botSpeed == undefined || game.WhoseTurn === 1 || !game.TurnOnBots) return

        const interval = setInterval(() => {
            if (game.IsBettingPhase) {
                autoBet(game)
            } else {
                if (difficulty === "Easy") {
                    autoPlayCard(game)
                } else if (difficulty === "Medium") {
                    autoPlayCardV2(game)
                }
            }
        }, botSpeed * 1000);

        return () => clearInterval(interval)
    })

   
</script>


<div class="flex flex-col gap-10 w-full h-screen  items-center overflow-auto">
    <div class="flex justify-end w-full p-3 gap-4">
        {#if !loggedIn}
        <Dialog.Root open={closeDialogue}>
        <Dialog.Trigger>
            <LogIn />
        </Dialog.Trigger>
        <Dialog.Content class="w-[320px]">
            <Dialog.Header>
            <Dialog.Title class="text-center mb-2">Login</Dialog.Title>
            <form class="flex flex-col gap-2 mb-2">
                <div class="flex justify-between gap-4">
                    <Label class="w-[60px] pr-6" for="username">Username</Label>
                    <Input bind:value={username} />
                </div>
                <div class="flex justify-between gap-4">
                    <Label class="w-[60px] pr-6" for="password">Password</Label>
                    <Input bind:value={password} />
                </div>
            </form>
            <div class="flex justify-center">
                <Button class="w-[80px]" 
                onclick={()=>{
                    loggedIn = true; 
                    closeDialogue = true}}>
                    Login
                </Button>
            </div>
            </Dialog.Header>
        </Dialog.Content>
        </Dialog.Root>
        {/if}

        <Popover.Root>
            <Popover.Trigger><Info /></Popover.Trigger>
            <Popover.Content class="w-auto mr-2 mt-1">
                <div>
                    <p>Trump Suit {game.Trump}</p>
                    <p>Bet Size {game.BetSize}</p>
                     {#if !game.IsBettingPhase}
                     <Separator />
                    <p>Bet Winner Player {game.BetWinner.ID}</p>
                    <p>Partner Card {game.PartnerCard.Rank} {game.PartnerCard.Suit}</p>
                    <Separator />
                    <p>Team 1 needs {6 + game.BetSize}</p>
                    <p>Team 2 needs {8 - game.BetSize}</p>
                    {/if}
                </div>
            </Popover.Content>
        </Popover.Root>
        <Popover.Root>
            <Popover.Trigger><Settings /></Popover.Trigger>
            <Popover.Content class="border-2 w-auto mr-1 mt-1">
                <div class="flex flex-col gap-4">
                    <div class="flex justify-between gap-3">
                        <Label for="difficulty">Difficulty</Label>
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
                    <div class="flex justify-between gap-3">
                        <Label for="bot-speed">Bot Time per Move</Label>
                        <Input type="number" bind:value={botSpeed} class="w-[100px]"/>
                    </div>
                    <div class="flex justify-between gap-3">
                        <Label for="hidden-mode">Hidden Mode </Label>
                        <div class="w-25">
                            <Switch id="hidden-mode" bind:checked={hiddenMode}/>
                        </div>
                    </div>
                    <div class="flex justify-between gap-3">
                        <Label for="hidden-mode">Bots </Label>
                        <div class="w-25">
                            <Switch id="bots" bind:checked={game.TurnOnBots}/>
                        </div>
                    </div>
                </div>
            </Popover.Content>
        </Popover.Root>

    </div>
    <div class="text-3xl">
        <p>Player {game.WhoseTurn}'s turn</p>
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
    <!-- MAIN PHASE -->
    <div class="flex flex-col gap-10">
        {#each game.Players as player}
        <div>
            <div class="flex gap-2">
                <p>Player {player.ID} ({player.Sets} sets) </p>
                {#if !hiddenMode && player.Partner !== null}
                <p>| Partner is Player {player.Partner?.ID}</p>
                {/if}
            </div>
            
            <div class="flex h-[100px]">
                {#each !hiddenMode || player.ID === 1 ? player.Cards : []  as card, index}
                <button
                    disabled={isCardIllegal(game, player, card)}
                    onclick={()=>playCard(game, card, player)}>
                    <HandDisplay index={index}>
                        <PokerCard card={card} isIllegal={isCardIllegal(game, player, card)}/>
                    </HandDisplay>
                </button>
                {/each}
                {#if !hiddenMode}
                <Separator orientation="vertical" class="mx-10 h-full"/>
                    {#each player.PlayedCards as card, index}
                     <button 
                        disabled={true}>
                        <HandDisplay index={index}>
                            <PokerCard card={card} isIllegal={true}/>
                        </HandDisplay>
                    </button>
                    {/each}
                {/if}
            </div>
        </div>
        {/each}
    </div>


    {:else} 
    <!-- BETTING PHASE -->
        <div class="flex flex-col gap-10">
            {#each hiddenMode ? [game.Players[0]] : game.Players as player}
            <div class="flex flex-col h-[100px]">
                <p class="mb-2">Player {player.ID}</p>
                <div class="flex pl-4">
                    {#each !hiddenMode || player.ID === 1 ? player.Cards : []  as card, index}
                        <HandDisplay index={index}>
                            <PokerCard card={card} isIllegal={false}/>
                        </HandDisplay>
                    {/each}
                </div>
            </div>
            {/each}
        </div>

        <div class="flex flex-col justify-center gap-2">
            <div class="flex gap-2">
                <Input bind:value={betSize} class="w-[60px] text-center numberInput" type="number" placeholder="1-7"/>
        
                <Select.Root type="single" bind:value={bettedSuit}>
                <Select.Trigger class="w-[70px]">
                    <p class="text-xl">{suitToSymbol.get(bettedSuit)}</p>
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
                variant="destructive"
                onclick={()=>raiseBet(game, betSize, bettedSuit)}
                disabled={!isLegalRaise(game, betSize, bettedSuit)}>Raise</Button>
            </div>
        </div>
    {/if}
</div>

<Dialog.Root open={game.Winner !== ""}>
    <Dialog.Trigger>
        <LogIn />
    </Dialog.Trigger>
    <Dialog.Content>
        <Dialog.Header>
        <Dialog.Title>{game.Winner} Won!</Dialog.Title>
        <Dialog.Description>
            <p class="mb-4">
                {game.Winner} has won {game.Winner ===  "Team 1" ? 6 + game.BetSize : 8 - game.BetSize} sets to win the game!
            </p>

            <form action="" method="POST" class="flex flex-col items-end" use:enhance>
                <input type="hidden" name="username" bind:value={username}>
                <input type="hidden" name="password" bind:value={password}>

                <Form.Button class="w-[60px]">
                    Save
                </Form.Button>
            </form>
        </Dialog.Description>
        </Dialog.Header>
    </Dialog.Content>
</Dialog.Root>
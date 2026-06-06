<script lang="ts">
    import * as Select from "$lib/components/ui/select/index.js";
    import { Input } from "$lib/components/ui/input/index.js";
    import { Button } from "$lib/components/ui/button/index.js";
    import PokerCard from "$lib/components/poker-card.svelte";
    import HandDisplay from "$lib/components/hand-display.svelte";
    import { suitToSymbol } from "$lib/utils"
    import { isLegalRaise } from "$lib/game/betting";
    import type { Game } from "$lib/game/types"

    let {
        game = {} as Game,
        humanSeat = 0,
        humanPlayerId = 0,
        hiddenMode = true,
        onRaise = (_bs: number, _suit: string) => {},
        onPass = () => {},
    } = $props()

    let betSize: number = $state(1)
    let bettedSuit: string = $state("Club")

    const playerIDToColor = new Map<number, string>([
        [1, "[var(--red)]"],
        [2, "[var(--blue)]"],
        [3, "[var(--yellow)]"],
        [4, "[var(--green)]"],
    ])

    function playerName(playerId: number): string {
        return game.Players?.find((p: any) => p.ID === playerId)?.Username ?? `P${playerId}`
    }
</script>

<!-- Moves display (last 3 betting actions) -->
<div class="flex flex-col items-center gap-1">
    {#each game.Moves.slice(-3) as move}
        <div class="flex items-center gap-2 {move === game.Moves[game.Moves.length - 1] ? 'text-base font-medium' : 'text-xs text-muted-foreground/60'}">
            <span class="text-{playerIDToColor.get(move.PlayerID)}">{playerName(move.PlayerID)}</span>
            {#if move.CardPlayed.Value === 0}
                <span>passed</span>
            {:else}
                <span>raised <strong>{move.CardPlayed.Value} {move.CardPlayed.Suit}</strong></span>
            {/if}
        </div>
    {/each}
    {#if game.Moves.length < 4}
        <div class="text-xs text-muted-foreground/40 mt-1">
            waiting for {playerName(game.WhoseTurn)}...
        </div>
    {/if}
    {#if game.Moves.length === 0}
        <div class="text-xs text-muted-foreground/40">
            {playerName(game.WhoseTurn)} to bet
        </div>
    {/if}
</div>

<!-- Player hands during betting -->
<div class="flex flex-col gap-10">
    {#each hiddenMode ? [game.Players[humanSeat]] : game.Players as player}
    <div class="flex flex-col h-[100px]">
        <p class="mb-2 text-{playerIDToColor.get(player.ID)}">{player.Username}</p>
        <div class="flex pl-4">
            {#each !hiddenMode || player.ID === humanPlayerId ? player.Cards : []  as card, index}
                <HandDisplay index={index}>
                    <PokerCard card={card} isIllegal={false} minify={false}/>
                </HandDisplay>
            {/each}
        </div>
    </div>
    {/each}
</div>

<!-- Raise / Pass controls -->
<div class="flex flex-col justify-center gap-2">
    <div class="flex flex-col gap-2 items-start w-[45%]">
        <div class="flex gap-2 w-full">
            <Input bind:value={betSize} class="text-center numberInput flex-1" type="number" min={1} max={7} placeholder="1-7"/>
            <Select.Root type="single" bind:value={bettedSuit}>
            <Select.Trigger class="flex-[3]">
                <p class="text-sm">{suitToSymbol.get(bettedSuit)} {bettedSuit}</p>
            </Select.Trigger>
            <Select.Content>
                <Select.Item value="Club">&clubs; Club</Select.Item>
                <Select.Item value="Diamond">&diams; Diamond</Select.Item>
                <Select.Item value="Heart">&hearts; Heart</Select.Item>
                <Select.Item value="Spades">&spades; Spades</Select.Item>
            </Select.Content>
            </Select.Root>
        </div>
        <div class="flex gap-2 w-full">
            <Button class="cursor-pointer flex-1" onclick={onPass}>Pass</Button>
            <Button 
            variant="destructive"
            onclick={() => onRaise(betSize, bettedSuit)}
            disabled={!isLegalRaise(game, betSize, bettedSuit)}
            class="bg-red-500 cursor-pointer flex-1 disabled:cursor-not-allowed disabled:opacity-50"
            >Raise</Button>
        </div>
    </div>
</div>

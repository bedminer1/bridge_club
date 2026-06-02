<script lang="ts">
    import { Separator } from "$lib/components/ui/separator/index.js";
    import PokerCard from "$lib/components/poker-card.svelte";
    import HandDisplay from "$lib/components/hand-display.svelte";
    import { isCardIllegal } from "$lib/game/legality";
    import type { Game } from "$lib/game/types"

    let {
        game = {} as Game,
        humanSeat = 0,
        humanPlayerId = 0,
        hiddenMode = true,
        onPlayCard = (_card: any, _player: any) => {},
    } = $props()

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

<!-- Current trick cards -->
<div class="flex justify-between w-full min-h-28 relative">
    <div class="flex gap-1.5 sm:gap-2 ml-2 sm:ml-4 items-start flex-shrink-0">
        {#each game.Moves as move, i}
        <div class="flex flex-col {i % 2 === 0 ? 'items-center mb-8' : 'items-center mt-8'}">
            <PokerCard card={move.CardPlayed} isIllegal={false} minify={false} />
            <p class="text-{playerIDToColor.get(move.PlayerID)} text-2xs sm:text-xs whitespace-nowrap">{playerName(move.PlayerID)}</p>
        </div>
        {/each}
    </div>

    {#if game.PreviousMoves.length !== 0} 
    <div class="relative mr-7 flex-shrink-0" style="width: {game.PreviousMoves.length * 12 + 40}px; min-height: {game.PreviousMoves.length * 8 + 40}px;">
        {#each game.PreviousMoves as move, i}
            <div class="absolute flex-col items-center gap-4" style="top: {i * 16}px; left: {i * 16}px; z-index: {i};">
                <p class="text-2xs text-right text-{playerIDToColor.get(move.PlayerID)} whitespace-nowrap">P{move.PlayerID}</p>
                <div>
                    <PokerCard card={move.CardPlayed} isIllegal={false} minify={true} />
                </div>
            </div>
        {/each}
    </div>
    {/if}
</div>

<!-- MAIN PHASE: player hands -->
<div class="flex flex-col gap-10">
    {#each hiddenMode ? [game.Players[humanSeat]] : game.Players as player}
    <div>
        <div class="flex gap-2">
            <p class="text-{playerIDToColor.get(player.ID)}">{player.Username} ({player.Sets} sets) </p>
            {#if !hiddenMode && player.Partner !== null}
            <p>| Partner is {playerName(player.Partner?.ID ?? 0)}</p>
            {/if}
        </div>
        
        <div class="flex h-[100px] pl-4">
            {#each player.Cards  as card, index}
            <button
                disabled={isCardIllegal(game, player, card)}
                onclick={() => onPlayCard(card, player)}
                class="text-left">
                <HandDisplay index={index}>
                    <PokerCard card={card} isIllegal={isCardIllegal(game, player, card)} minify={false}/>
                </HandDisplay>
            </button>
            {/each}
            {#if !hiddenMode}
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

    {#if hiddenMode}
    <div class="flex gap-4">
        {#each game.Players.slice(1, 4) as player, index}
            <div class="flex flex-col gap-0">
            <p class="text-{playerIDToColor.get(player.ID)}">{player.Username}</p>
            <p class="text-sm text-muted-foreground">({player.Sets} sets)</p>
            </div>

            {#if index < 2}
            <Separator orientation="vertical" />
            {/if}
        {/each}
    </div>
    {/if}
</div>

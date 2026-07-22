<script lang="ts">
    import CardArt from "$lib/components/card-art.svelte";
    import { isCardIllegal } from "$lib/game/legality";
    import { wsClient } from "$lib/game/ws-client";
    import { playerIDToColor, playerName } from "$lib/game/player-utils";
    import { frontendCardToApiCard } from "$lib/game/api-game";
    import type { Game } from "$lib/game/types"

    let {
        game = {} as Game,
        humanSeat = 0,
        humanPlayerId = 0,
        hiddenMode = true,
        disabled = false,
        roomId = "",
    } = $props()

    function playCard(card: any) {
        if (!roomId || disabled) return
        wsClient.gameAction("play", undefined, frontendCardToApiCard(card))
    }

    // Map players to table positions relative to human
    let tablePlayers = $derived.by(() => {
        if (!game.Players || game.Players.length < 4) return { south: null, west: null, north: null, east: null }
        const idx = game.Players.findIndex(p => p.ID === humanPlayerId)
        if (idx === -1) return { south: null, west: null, north: null, east: null }
        return {
            south: game.Players[idx],
            west: game.Players[(idx + 1) % 4],
            north: game.Players[(idx + 2) % 4],
            east: game.Players[(idx + 3) % 4],
        }
    })

    // Last completed set (won trick stack, shown at bottom-left)
    let lastSet = $derived(game.CompletedSets?.[game.CompletedSets?.length - 1] ?? null)

    /** Static card back HTML — fully controlled, no XSS risk */
    const CARD_BACK = '<div class="w-[35px] sm:w-[40px] rounded-sm bg-gradient-to-br from-indigo-950 via-blue-950 to-slate-900 border-2 border-indigo-800/40 shadow-inner flex items-center justify-center relative overflow-hidden flex-shrink-0" style="aspect-ratio: 2.5 / 3.5;"><div class="absolute inset-[2px] rounded-sm border border-indigo-500/15"></div><div class="absolute inset-[4px] rounded-sm border border-indigo-500/10"></div><span class="relative text-indigo-400/20 text-[8px] select-none">♠</span></div>'
</script>

<!-- ── Table Layout ──────────────────────────────── -->
<div class="relative flex flex-col items-center w-full max-w-5xl mx-auto gap-0.5 h-[540px] sm:h-[540px]">

    <!-- ── North (player across) ──────────────────── -->
    {#if tablePlayers.north}
    <div class="flex flex-col items-center flex-shrink-0">
        <div class="flex flex-col items-center mb-2">
            <p class="text-{playerIDToColor.get(tablePlayers.north.ID)} text-sm">{tablePlayers.north.Username}</p>
            <p class="text-{playerIDToColor.get(tablePlayers.north.ID)} text-sm">{tablePlayers.north.Sets} sets</p>        
        </div>
        
        <div class="flex items-end justify-center" style="min-height: 42px;">
            {#each tablePlayers.north.Cards as card, i}
                <div class="relative" style="margin-left: {i === 0 ? '0' : '-1.25rem'}; z-index: {i};">
                    {#if hiddenMode && tablePlayers.north.ID !== humanPlayerId}
                        {@html CARD_BACK}
                    {:else}
                        <CardArt card={card} isIllegal={false} minify={true} />
                    {/if}
                </div>
            {/each}
        </div>
    </div>
    {/if}

    <!-- ── Middle: West | Center | East ──────────── -->
    <div class="flex w-full justify-between items-stretch gap-1 flex-1 mt-1">

        <!-- West (left opponent) — scrunched -->
        {#if tablePlayers.west}
        <div class="flex flex-col items-start gap-1 min-w-[9rem] mt-15">
            <div class="flex flex-col items-end">
                {#each tablePlayers.west.Cards as card, i}
                    <div class="relative" style="margin-top: {i === 0 ? '0' : '-2.6rem'}; z-index: {i};">
                        {#if hiddenMode && tablePlayers.west.ID !== humanPlayerId}
                            <div class="rotate-90">
                                {@html CARD_BACK}
                            </div>
                        {:else}
                            <CardArt card={card} isIllegal={false} minify={true} />
                        {/if}
                    </div>
                {/each}
            </div>
            <div class="flex flex-col items-start leading-tight">
                <p class="text-{playerIDToColor.get(tablePlayers.west.ID)} text-xs whitespace-nowrap">{tablePlayers.west.Username}</p>
                <p class="text-{playerIDToColor.get(tablePlayers.west.ID)} text-xs whitespace-nowrap">{tablePlayers.west.Sets} sets</p>
            </div>
        </div>
        {/if}

        <!-- Center: current trick + game info -->
        <div class="flex-1 flex flex-col items-center justify-center min-h-[80px]">
            {#if game.Moves.length > 0}
                <div class="flex items-center justify-center" style="gap: max(3rem, 1.2vw);">
                    {#each game.Moves as move, i}
                        <div class="flex flex-col items-center scale-[1.2] origin-center">
                            <CardArt card={move.CardPlayed} isIllegal={false} minify={false} />
                            <p class="text-{playerIDToColor.get(move.PlayerID)} text-2xs whitespace-nowrap">{playerName(game, move.PlayerID)}</p>
                        </div>
                    {/each}
                </div>
            {:else}
                <p class="mt-4 text-xs text-muted-foreground/40 italic">no cards played yet</p>
            {/if}
            <p class="mt-4 text-xs text-muted-foreground mt-1">{playerName(game, game.WhoseTurn)}'s turn</p>
        </div>

        <!-- East (right opponent) — scrunched -->
        {#if tablePlayers.east}
        <div class="flex flex-col items-end gap-1 min-w-[9rem] mt-15">
            <div class="flex flex-col items-start order-2">
                {#each tablePlayers.east.Cards as card, i}
                    <div class="relative" style="margin-top: {i === 0 ? '0' : '-2.6rem'}; z-index: {i};">
                        {#if hiddenMode && tablePlayers.east.ID !== humanPlayerId}
                            <div class="-rotate-90">
                                {@html CARD_BACK}
                            </div>
                        {:else}
                            <CardArt card={card} isIllegal={false} minify={true} />
                        {/if}
                    </div>
                {/each}
            </div>
            <div class="flex flex-col items-end leading-tight order-1">
                <p class="text-{playerIDToColor.get(tablePlayers.east.ID)} text-xs whitespace-nowrap text-right">{tablePlayers.east.Username}</p>
                <p class="text-{playerIDToColor.get(tablePlayers.east.ID)} text-xs whitespace-nowrap text-right">{tablePlayers.east.Sets} sets</p>
            </div>
        </div>
        {/if}
    </div>

    <!-- ── South (human) ──────────────────────────── -->
    {#if tablePlayers.south}
    <div class="flex flex-col items-center mt-0.5 flex-shrink-0">
        <div class="flex items-start justify-center">
            {#each tablePlayers.south.Cards as card, i}
                <button onclick={() => playCard(card)}
                    disabled={disabled || tablePlayers.south.ID !== humanPlayerId || isCardIllegal(game, tablePlayers.south, card)}
                    class="transition-transform hover:z-10 hover:-translate-y-3 active:-translate-y-2 disabled:grayscale disabled:brightness-90 disabled:hover:translate-y-0 disabled:cursor-not-allowed scale-[1.2]"
                    style="margin-left: {i === 0 ? '0' : '-1.5rem'}; z-index: {i};">
                    <CardArt card={card} isIllegal={isCardIllegal(game, tablePlayers.south, card)} minify={false} />
                </button>
            {/each}
        </div>
        <div class="flex flex-col items-center mt-2">
            <p class="text-{playerIDToColor.get(tablePlayers.south.ID)} text-sm">{tablePlayers.south.Username}</p>
            <p class="text-{playerIDToColor.get(tablePlayers.south.ID)} text-sm">{tablePlayers.south.Sets} sets</p>
        </div>
        
    </div>
    {/if}
<!-- ── Won trick stack (top-left) ── -->
{#if lastSet}
    <div class="absolute top-2 left-2 z-10 flex gap-0 p-1.5 rounded-lg bg-card/70 backdrop-blur-sm border border-border/50 shadow-sm">
        <div class="flex gap-0 items-end">
            {#each lastSet.Cards as tc, i}
                <div class="{i > 0 ? '-ml-3' : ''} opacity-80">
                    <CardArt card={tc} isIllegal={false} minify={true} />
                </div>
            {/each}
        </div>
    </div>
{/if}
</div>

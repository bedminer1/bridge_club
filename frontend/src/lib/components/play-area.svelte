<script lang="ts">
    import PokerCard from "$lib/components/poker-card.svelte";
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

    /** Fan angle for south cards. */
    function fanAngle(index: number, total: number): number {
        if (total <= 1) return 0
        const spread = 36
        const center = (total - 1) / 2
        return (index - center) * (spread / Math.max(total, 2))
    }

    /** More aggressive fan for north cards. */
    function fanAngleOther(index: number, total: number): number {
        if (total <= 1) return 0
        const spread = 52
        const center = (total - 1) / 2
        return (index - center) * (spread / Math.max(total, 2))
    }

    /** Fan angle for vertical columns (west, east) — scrunched tighter. */
    function fanAngleVertical(index: number, total: number, dir: 'left' | 'right'): number {
        if (total <= 1) return 0
        const spread = 32  // tighter than before
        const center = (total - 1) / 2
        const angle = (index - center) * (spread / Math.max(total, 2))
        return dir === 'left' ? -angle : angle
    }

    /** Static card back HTML — fully controlled, no XSS risk */
    const CARD_BACK = '<div class="w-[35px] h-[40px] rounded-sm bg-gradient-to-br from-indigo-950 via-blue-950 to-slate-900 border-2 border-indigo-800/40 shadow-inner flex items-center justify-center relative overflow-hidden flex-shrink-0"><div class="absolute inset-[2px] rounded-sm border border-indigo-500/15"></div><div class="absolute inset-[4px] rounded-sm border border-indigo-500/10"></div><span class="relative text-indigo-400/20 text-[8px] select-none">♠</span></div>'
</script>

<!-- ── Table Layout ──────────────────────────────── -->
<div class="relative flex flex-col items-center w-full max-w-4xl mx-auto gap-0.5 min-h-[320px] sm:min-h-[400px]">

    <!-- ── North (player across) ──────────────────── -->
    {#if tablePlayers.north}
    <div class="flex flex-col items-center flex-shrink-0">
        <p class="text-{playerIDToColor.get(tablePlayers.north.ID)} text-sm">{tablePlayers.north.Username} ({tablePlayers.north.Sets})</p>
        <div class="flex items-end justify-center" style="min-height: 42px;">
            {#each tablePlayers.north.Cards as card, i}
                {@const a = -fanAngleOther(i, tablePlayers.north.Cards.length)}
                <div class="relative" style="margin-left: {i === 0 ? '0' : '-1.25rem'}; transform: rotate({a}deg); transform-origin: top center; z-index: {i};">
                    {#if hiddenMode && tablePlayers.north.ID !== humanPlayerId}
                        {@html CARD_BACK}
                    {:else}
                        <PokerCard card={card} isIllegal={false} minify={true} />
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
        <div class="flex items-center gap-0.5">
            <div class="flex flex-col items-end">
                {#each tablePlayers.west.Cards as card, i}
                    {@const a = fanAngleVertical(i, tablePlayers.west.Cards.length, 'right')}
                    <div class="relative" style="margin-top: {i === 0 ? '0' : '-2rem'}; transform: rotate({a}deg); transform-origin: right center; z-index: {i};">
                        {#if hiddenMode && tablePlayers.west.ID !== humanPlayerId}
                            {@html CARD_BACK}
                        {:else}
                            <PokerCard card={card} isIllegal={false} minify={true} />
                        {/if}
                    </div>
                {/each}
            </div>
            <p class="text-{playerIDToColor.get(tablePlayers.west.ID)} text-xs whitespace-nowrap">{tablePlayers.west.Username} ({tablePlayers.west.Sets})</p>
        </div>
        {/if}

        <!-- Center: current trick + game info -->
        <div class="flex-1 flex flex-col items-center justify-center min-h-[80px]">
            {#if game.Moves.length > 0}
                <div class="flex gap-1 sm:gap-1.5 items-center justify-center">
                    {#each game.Moves as move, i}
                        <div class="flex flex-col items-center">
                            <PokerCard card={move.CardPlayed} isIllegal={false} minify={false} />
                            <p class="text-{playerIDToColor.get(move.PlayerID)} text-2xs whitespace-nowrap">{playerName(game, move.PlayerID)}</p>
                        </div>
                    {/each}
                </div>
            {:else}
                <p class="text-xs text-muted-foreground/40 italic">no cards played yet</p>
            {/if}
            <p class="text-xs text-muted-foreground mt-1">{playerName(game, game.WhoseTurn)}'s turn</p>
        </div>

        <!-- East (right opponent) — scrunched -->
        {#if tablePlayers.east}
        <div class="flex items-center gap-0.5">
            <div class="flex flex-col items-end order-1">
                <p class="text-{playerIDToColor.get(tablePlayers.east.ID)} text-xs whitespace-nowrap text-right">{tablePlayers.east.Username} ({tablePlayers.east.Sets})</p>
            </div>
            <div class="flex flex-col items-start order-2">
                {#each tablePlayers.east.Cards as card, i}
                    {@const a = fanAngleVertical(i, tablePlayers.east.Cards.length, 'left')}
                    <div class="relative" style="margin-top: {i === 0 ? '0' : '-2rem'}; transform: rotate({a}deg); transform-origin: left center; z-index: {i};">
                        {#if hiddenMode && tablePlayers.east.ID !== humanPlayerId}
                            {@html CARD_BACK}
                        {:else}
                            <PokerCard card={card} isIllegal={false} minify={true} />
                        {/if}
                    </div>
                {/each}
            </div>
        </div>
        {/if}
    </div>

    <!-- ── South (human) ──────────────────────────── -->
    {#if tablePlayers.south}
    <div class="flex flex-col items-center mt-0.5 flex-shrink-0">
        <div class="flex items-start justify-center">
            {#each tablePlayers.south.Cards as card, i}
                {@const angle = fanAngle(i, tablePlayers.south.Cards.length)}
                <button onclick={() => playCard(card)}
                    disabled={disabled || tablePlayers.south.ID !== humanPlayerId || isCardIllegal(game, tablePlayers.south, card)}
                    class="transition-transform hover:z-10 hover:-translate-y-3 active:-translate-y-2 disabled:grayscale disabled:brightness-90 disabled:hover:translate-y-0 disabled:cursor-not-allowed"
                    style="margin-left: {i === 0 ? '0' : '-1.5rem'}; transform: rotate({angle}deg); transform-origin: bottom center; z-index: {i};">
                    <PokerCard card={card} isIllegal={isCardIllegal(game, tablePlayers.south, card)} minify={false} />
                </button>
            {/each}
        </div>
        <p class="text-{playerIDToColor.get(tablePlayers.south.ID)} text-sm">{tablePlayers.south.Username} ({tablePlayers.south.Sets})</p>
    </div>
    {/if}
<!-- ── Won trick stack (top-left) ── -->
{#if lastSet}
    <div class="absolute top-2 left-2 z-10 flex gap-0 p-1.5 rounded-lg bg-card/70 backdrop-blur-sm border border-border/50 shadow-sm">
        <div class="flex gap-0 items-end">
            {#each lastSet.Cards as tc, i}
                <div class="{i > 0 ? '-ml-3' : ''} opacity-80">
                    <PokerCard card={tc} isIllegal={false} minify={true} />
                </div>
            {/each}
        </div>
    </div>
{/if}
</div>

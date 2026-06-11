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

    // Last completed set + which table position won it
    let lastSet = $derived(game.CompletedSets?.[game.CompletedSets?.length - 1] ?? null)
    let lastSetPosition = $derived.by(() => {
        if (!lastSet || !tablePlayers.south) return null
        for (const [pos, p] of Object.entries(tablePlayers)) {
            if (p && p.ID === lastSet.WinnerID) return pos as 'south'|'west'|'north'|'east'
        }
        return null
    })

    /** Fan angle for horizontal rows (south, north). */
    function fanAngle(index: number, total: number): number {
        if (total <= 1) return 0
        const spread = 36
        const center = (total - 1) / 2
        return (index - center) * (spread / Math.max(total, 2))
    }

    /** More aggressive fan for other players' hidden/visible cards. */
    function fanAngleOther(index: number, total: number): number {
        if (total <= 1) return 0
        const spread = 52
        const center = (total - 1) / 2
        return (index - center) * (spread / Math.max(total, 2))
    }

    /** Fan angle for vertical columns (west, east). */
    function fanAngleVertical(index: number, total: number, dir: 'left' | 'right'): number {
        if (total <= 1) return 0
        const spread = 44
        const center = (total - 1) / 2
        const angle = (index - center) * (spread / Math.max(total, 2))
        return dir === 'left' ? -angle : angle
    }

    const CARD_BACK = '<div class="w-[43px] h-[52px] sm:w-[50px] sm:h-[60px] rounded-sm bg-gradient-to-br from-indigo-950 via-blue-950 to-slate-900 border-2 border-indigo-800/40 shadow-inner flex items-center justify-center relative overflow-hidden flex-shrink-0"><div class="absolute inset-[3px] rounded-sm border border-indigo-500/15"></div><div class="absolute inset-[6px] rounded-sm border border-indigo-500/10"></div><span class="relative text-indigo-400/20 text-[10px] sm:text-xs select-none">♠</span></div>'
</script>

<!-- ── Table Layout ──────────────────────────────── -->
<div class="flex flex-col items-center w-full max-w-4xl mx-auto gap-0.5">

    <!-- ── North (player across) ──────────────────── -->
    {#if tablePlayers.north}
    <div class="flex flex-col items-center">
        <p class="text-{playerIDToColor.get(tablePlayers.north.ID)} text-xs">{tablePlayers.north.Username} ({tablePlayers.north.Sets} sets)</p>
        <div class="flex items-end justify-center relative" style="min-height: 48px;">
            {#each tablePlayers.north.Cards as card, i}
                {@const a = -fanAngleOther(i, tablePlayers.north.Cards.length)}
                <div class="relative" style="margin-left: {i === 0 ? '0' : '-1.5rem'}; transform: rotate({a}deg); transform-origin: top center; z-index: {i};">
                    {#if hiddenMode && tablePlayers.north.ID !== humanPlayerId}
                        {@html CARD_BACK}
                    {:else}
                        <PokerCard card={card} isIllegal={false} minify={false} />
                    {/if}
                </div>
            {/each}
            <!-- Last trick won by north -->
            {#if lastSetPosition === 'north' && lastSet}
                <div class="absolute -bottom-1 left-1/2 -translate-x-1/2 flex gap-0">
                    {#each lastSet.Cards as tc}
                        <div class="-ml-2 first:ml-0 opacity-70">
                            <PokerCard card={tc} isIllegal={false} minify={true} />
                        </div>
                    {/each}
                </div>
            {/if}
        </div>
    </div>
    {/if}

    <!-- ── Middle: West | Center | East ──────────── -->
    <div class="flex w-full justify-between items-stretch gap-1 mt-1">

        <!-- West (left opponent) -->
        {#if tablePlayers.west}
        <div class="flex items-center gap-1">
            <div class="flex flex-col items-end relative" style="min-width: 45px;">
                {#each tablePlayers.west.Cards as card, i}
                    {@const a = fanAngleVertical(i, tablePlayers.west.Cards.length, 'right')}
                    <div class="relative" style="margin-top: {i === 0 ? '0' : '-1.5rem'}; transform: rotate({a}deg); transform-origin: right center; z-index: {i};">
                        {#if hiddenMode && tablePlayers.west.ID !== humanPlayerId}
                            {@html CARD_BACK}
                        {:else}
                            <PokerCard card={card} isIllegal={false} minify={false} />
                        {/if}
                    </div>
                {/each}
                <!-- Last trick won by west -->
                {#if lastSetPosition === 'west' && lastSet}
                    <div class="absolute -right-1 top-1/2 -translate-y-1/2 flex flex-col gap-0">
                        {#each lastSet.Cards as tc}
                            <div class="-mt-2 first:mt-0 opacity-70">
                                <PokerCard card={tc} isIllegal={false} minify={true} />
                            </div>
                        {/each}
                    </div>
                {/if}
            </div>
            <p class="text-{playerIDToColor.get(tablePlayers.west.ID)} text-xs whitespace-nowrap">{tablePlayers.west.Username} {tablePlayers.west.Sets}</p>
        </div>
        {/if}

        <!-- Center: current trick + game info -->
        <div class="flex-1 flex flex-col items-center justify-center min-h-[100px]">
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

            <!-- Previous trick in center (if winner is south or unknown) -->
            {#if lastSet && (!lastSetPosition || lastSetPosition === 'south')}
                <div class="flex gap-0 mt-1 opacity-60">
                    {#each lastSet.Cards as tc}
                        <div class="-ml-3 first:ml-0">
                            <PokerCard card={tc} isIllegal={false} minify={true} />
                        </div>
                    {/each}
                </div>
            {/if}

            <p class="text-xs text-muted-foreground mt-1">{playerName(game, game.WhoseTurn)}'s turn</p>
        </div>

        <!-- East (right opponent) -->
        {#if tablePlayers.east}
        <div class="flex items-center gap-1">
            <p class="text-{playerIDToColor.get(tablePlayers.east.ID)} text-xs whitespace-nowrap">{tablePlayers.east.Username} {tablePlayers.east.Sets}</p>
            <div class="flex flex-col items-start relative" style="min-width: 45px;">
                {#each tablePlayers.east.Cards as card, i}
                    {@const a = fanAngleVertical(i, tablePlayers.east.Cards.length, 'left')}
                    <div class="relative" style="margin-top: {i === 0 ? '0' : '-1.5rem'}; transform: rotate({a}deg); transform-origin: left center; z-index: {i};">
                        {#if hiddenMode && tablePlayers.east.ID !== humanPlayerId}
                            {@html CARD_BACK}
                        {:else}
                            <PokerCard card={card} isIllegal={false} minify={false} />
                        {/if}
                    </div>
                {/each}
                <!-- Last trick won by east -->
                {#if lastSetPosition === 'east' && lastSet}
                    <div class="absolute -left-1 top-1/2 -translate-y-1/2 flex flex-col gap-0">
                        {#each lastSet.Cards as tc}
                            <div class="-mt-2 first:mt-0 opacity-70">
                                <PokerCard card={tc} isIllegal={false} minify={true} />
                            </div>
                        {/each}
                    </div>
                {/if}
            </div>
        </div>
        {/if}
    </div>

    <!-- ── South (human) ──────────────────────────── -->
    {#if tablePlayers.south}
    <div class="flex flex-col items-center mt-1">
        <div class="flex items-start justify-center pb-1">
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
        <p class="text-{playerIDToColor.get(tablePlayers.south.ID)} text-xs">{tablePlayers.south.Username} ({tablePlayers.south.Sets} sets)</p>
    </div>
    {/if}
</div>

<!-- Trick cards that belong to west/north/east are shown near their position above -->
